//! Read-only USB/sysfs inventory and bounded HID++ framing.
use llc_core::Model;
use serde::Serialize;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub const MAX_REPORT_BYTES: usize = 64;
pub const MAX_DESCRIPTOR_BYTES: usize = 4096;
#[derive(Debug, Serialize)]
pub struct Node {
    pub path: String,
    pub interface: Option<String>,
    pub readable: bool,
    pub descriptor_hex: Option<String>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Device {
    pub model: Model,
    pub vendor_id: String,
    pub product_id: String,
    pub usb_port: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    /// bcdDevice is a USB revision, not a proven firmware version.
    pub usb_device_revision: Option<String>,
    pub firmware: Option<String>,
    pub serial_present: bool,
    pub input_nodes: Vec<Node>,
    pub hidraw_nodes: Vec<Node>,
    pub qualification: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub read_only: bool,
    pub kernel: String,
    pub devices: Vec<Device>,
    pub hardware_gate: &'static str,
    pub limitations: Vec<&'static str>,
}
fn text(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_owned())
}
fn bounded(path: &Path, max: usize) -> io::Result<Vec<u8>> {
    let mut bytes = vec![];
    fs::File::open(path)?
        .take((max + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sysfs value exceeds capture bound",
        ));
    }
    Ok(bytes)
}
pub fn probe() -> io::Result<Report> {
    probe_at(
        Path::new("/sys"),
        Path::new("/dev"),
        text("/proc/sys/kernel/osrelease").unwrap_or_default(),
    )
}

/// Alternate roots are available to tests; privileged IPC never accepts paths.
pub fn probe_at(sys: &Path, dev: &Path, kernel: String) -> io::Result<Report> {
    let mut devices = vec![];
    let entries = match fs::read_dir(sys.join("bus/usb/devices")) {
        Ok(entries) => Some(entries),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    for entry in entries.into_iter().flatten() {
        let entry = entry?;
        let usb = entry.path();
        if text(usb.join("idVendor")).as_deref() != Some("046d") {
            continue;
        }
        let product_id = text(usb.join("idProduct")).unwrap_or_default();
        let model = match product_id.as_str() {
            "c08b" => Model::G502Hero,
            "c33f" => Model::G815,
            _ => continue,
        };
        let physical = fs::canonicalize(&usb)?;
        let collect = |class: &str, prefix: &str| -> io::Result<Vec<Node>> {
            let mut nodes = vec![];
            let entries = match fs::read_dir(sys.join("class").join(class)) {
                Ok(entries) => entries,
                Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(nodes),
                Err(e) => return Err(e),
            };
            for node in entries {
                let node = node?;
                let name = node.file_name().to_string_lossy().into_owned();
                if !name.starts_with(prefix) {
                    continue;
                }
                let Ok(path) = fs::canonicalize(node.path().join("device")) else {
                    continue;
                };
                if !path.starts_with(&physical) {
                    continue;
                }
                // Only descendants of this exact USB device, never sibling passthrough devices.
                let belongs = path
                    .ancestors()
                    .find(|p| p.join("idVendor").is_file() && p.join("idProduct").is_file());
                if belongs != Some(physical.as_path()) {
                    continue;
                }
                let interface = path
                    .ancestors()
                    .find_map(|p| text(p.join("bInterfaceNumber")));
                let node_path = if class == "input" {
                    dev.join("input").join(&name)
                } else {
                    dev.join(&name)
                };
                let readable = fs::File::open(&node_path).is_ok();
                let descriptor = if class == "hidraw" {
                    path.ancestors()
                        .find(|p| p.starts_with(&physical) && p.join("report_descriptor").is_file())
                        .map(|p| bounded(&p.join("report_descriptor"), MAX_DESCRIPTOR_BYTES))
                } else {
                    None
                };
                let (descriptor_hex, error) = match descriptor {
                    Some(Ok(bytes)) => (
                        Some(bytes.iter().map(|b| format!("{b:02x}")).collect()),
                        None,
                    ),
                    Some(Err(e)) => (None, Some(e.to_string())),
                    None => (None, None),
                };
                nodes.push(Node {
                    path: node_path.display().to_string(),
                    interface,
                    readable,
                    descriptor_hex,
                    error,
                });
            }
            nodes.sort_by(|a, b| a.path.cmp(&b.path));
            Ok(nodes)
        };
        devices.push(Device {
            model,
            vendor_id: "046d".into(),
            product_id,
            usb_port: entry.file_name().to_string_lossy().into_owned(),
            manufacturer: text(usb.join("manufacturer")),
            product: text(usb.join("product")),
            usb_device_revision: text(usb.join("bcdDevice")),
            firmware: None,
            serial_present: text(usb.join("serial")).is_some(),
            input_nodes: collect("input", "event")?,
            hidraw_nodes: collect("hidraw", "hidraw")?,
            qualification: "untested",
        });
    }
    devices.sort_by(|a, b| a.usb_port.cmp(&b.usb_port));
    Ok(Report { schema_version: 1, read_only: true, kernel, devices, hardware_gate: "G0-unverified", limitations: vec!["Inventory and descriptors only; no input event recording, feature requests or mode writes.", "Firmware, control routes, suppression, DPI and lighting remain unverified.", "Serial numbers are omitted; USB topology is included for node association."] })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidppReport {
    pub device: u8,
    pub feature: u8,
    pub function: u8,
    pub software_id: u8,
    pub payload: Vec<u8>,
}
impl HidppReport {
    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        let expected = match bytes.first() {
            Some(0x10) => 7,
            Some(0x11) => 20,
            Some(0x12) => 64,
            _ => return Err("unknown report id"),
        };
        if bytes.len() != expected || bytes.len() > MAX_REPORT_BYTES {
            return Err("incorrect report length");
        }
        Ok(Self {
            device: bytes[1],
            feature: bytes[2],
            function: bytes[3] >> 4,
            software_id: bytes[3] & 15,
            payload: bytes[4..].to_vec(),
        })
    }
    pub fn matches(&self, request: &Self) -> bool {
        self.software_id != 0
            && self.device == request.device
            && self.feature == request.feature
            && self.function == request.function
            && self.software_id == request.software_id
    }
}

pub fn descriptor_path(root: &Path, relative: &str) -> PathBuf {
    root.join(relative)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framing_rejects_short_and_malformed_reports() {
        for length in 0..70 {
            let mut bytes = vec![0; length];
            if length > 0 {
                bytes[0] = 0x11;
            }
            assert_eq!(HidppReport::decode(&bytes).is_ok(), length == 20);
        }
        assert!(HidppReport::decode(&[0xff; 20]).is_err());
    }
    #[test]
    fn notification_is_not_a_transaction_reply() {
        let request = HidppReport::decode(&[0x10, 0xff, 1, 0x17, 0, 0, 0]).unwrap();
        let mut reply = request.clone();
        assert!(reply.matches(&request));
        reply.software_id = 0;
        assert!(!reply.matches(&request));
        reply.software_id = 7;
        reply.device = 0;
        assert!(!reply.matches(&request));
    }
    #[test]
    fn inventory_filters_usb_hub_siblings() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let base = std::env::temp_dir().join(format!("llc-probe-{}", std::process::id()));
            let sys = base.join("sys");
            let usb = sys.join("devices/usb1/1-1");
            let interface = usb.join("1-1:1.0");
            fs::create_dir_all(interface.join("input/input1")).unwrap();
            fs::create_dir_all(sys.join("bus/usb/devices")).unwrap();
            fs::create_dir_all(sys.join("class/input/event1")).unwrap();
            fs::create_dir_all(base.join("dev/input")).unwrap();
            fs::write(usb.join("idVendor"), "046d").unwrap();
            fs::write(usb.join("idProduct"), "c33f").unwrap();
            fs::write(interface.join("bInterfaceNumber"), "00").unwrap();
            symlink(&usb, sys.join("bus/usb/devices/1-1")).unwrap();
            symlink(
                interface.join("input/input1"),
                sys.join("class/input/event1/device"),
            )
            .unwrap();
            let report = probe_at(&sys, &base.join("dev"), "test".into()).unwrap();
            assert_eq!(report.devices.len(), 1);
            assert_eq!(report.devices[0].input_nodes.len(), 1);
            assert!(!report.devices[0].input_nodes[0].readable);
            fs::remove_dir_all(base).unwrap();
        }
    }
}
