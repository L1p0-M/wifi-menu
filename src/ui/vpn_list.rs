use gtk4::prelude::*;
use gtk4::{self as gtk, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use crate::dbus::network_manager::VpnProfile;

#[derive(Clone)]
pub struct VpnList {
    container: gtk::Box,
    list_box: gtk::Box,
    public_ip_label: gtk::Label,
    isp_label: gtk::Label,
    dns_label: gtk::Label,
    profiles: Rc<RefCell<Vec<VpnProfile>>>,
    on_toggle: Rc<RefCell<Option<Rc<dyn Fn(String, bool)>>>>,
}

impl VpnList {
    pub fn new() -> Self {
        let container = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .spacing(16)
            .build();

        // Privacy Dashboard Header
        let dashboard = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["orbit-vpn-dashboard"])
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .build();

        let dash_title = gtk::Label::builder()
            .label("PRIVACY DASHBOARD")
            .css_classes(["orbit-section-header"])
            .halign(gtk::Align::Start)
            .build();
        dashboard.append(&dash_title);

        let info_grid = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build();

        // IP & ISP Row
        let ip_row = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        let ip_icon = gtk::Image::builder()
            .icon_name("network-vpn-symbolic")
            .pixel_size(24)
            .css_classes(["orbit-icon-accent"])
            .build();
        
        let ip_info = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        let public_ip_label = gtk::Label::builder()
            .label("IP: Detecting...")
            .css_classes(["orbit-ssid"])
            .halign(gtk::Align::Start)
            .selectable(true)
            .build();
        
        let isp_label = gtk::Label::builder()
            .label("Direct Connection")
            .css_classes(["orbit-status"])
            .halign(gtk::Align::Start)
            .build();

        ip_info.append(&public_ip_label);
        ip_info.append(&isp_label);
        
        ip_row.append(&ip_icon);
        ip_row.append(&ip_info);
        info_grid.append(&ip_row);

        // DNS Row
        let dns_row = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        let dns_icon = gtk::Image::builder()
            .icon_name("web-browser-symbolic")
            .pixel_size(24)
            .css_classes(["orbit-signal-icon"])
            .build();
        
        let dns_info = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        let dns_title = gtk::Label::builder()
            .label("DNS SERVERS")
            .css_classes(["orbit-section-header"])
            .halign(gtk::Align::Start)
            .build();

        let dns_label = gtk::Label::builder()
            .label("Detecting DNS...")
            .css_classes(["orbit-status"])
            .halign(gtk::Align::Start)
            .wrap(true)
            .xalign(0.0)
            .selectable(true)
            .build();

        dns_info.append(&dns_title);
        dns_info.append(&dns_label);
        
        dns_row.append(&dns_icon);
        dns_row.append(&dns_info);
        info_grid.append(&dns_row);
        
        dashboard.append(&info_grid);
        container.append(&dashboard);

        // VPN Profiles Section
        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .css_classes(["orbit-scrolled"])
            .build();
        
        let list_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["orbit-list"])
            .build();
        
        scrolled.set_child(Some(&list_box));
        container.append(&scrolled);
        
        let list = Self {
            container,
            list_box,
            public_ip_label,
            isp_label,
            dns_label,
            profiles: Rc::new(RefCell::new(Vec::new())),
            on_toggle: Rc::new(RefCell::new(None)),
        };
        
        list.show_placeholder();
        list
    }

    pub fn set_privacy_info(&self, ip: &str, isp: &str, dns_servers: &[String], is_secure: bool) {
        log::info!("VpnList: Updating privacy info: IP={}, ISP={}", ip, isp);
        self.public_ip_label.set_label(&format!("IP: {}", ip));
        self.isp_label.set_label(isp);
        
        let ip_prefix = if ip.contains(':') {
            ip.split(':').take(4).collect::<Vec<_>>().join(":")
        } else {
            ip.split('.').take(3).collect::<Vec<_>>().join(".")
        };

        let mut dns_display = Vec::new();
        for dns in dns_servers {
            let provider = match dns.as_str() {
                "1.1.1.1" | "1.0.0.1" | "2606:4700:4700::1111" => "Cloudflare (Secure)",
                "8.8.8.8" | "8.8.4.4" | "2001:4860:4860::8888" => "Google",
                "9.9.9.9" | "149.112.112.112" | "2620:fe::fe" => "Quad9 (Secure)",
                d if d.contains(&ip_prefix) => "Local Router / ISP",
                _ => "External DNS",
            };
            dns_display.push(format!("{}\n{}", provider, dns));
        }

        if dns_display.is_empty() {
            self.dns_label.set_label("Default (System)");
        } else {
            self.dns_label.set_label(&dns_display.join("\n\n"));
        }

        if is_secure {
            self.isp_label.add_css_class("orbit-status-accent");
            self.isp_label.set_label(&format!("{} (Secure)", isp));
        } else {
            self.isp_label.remove_css_class("orbit-status-accent");
        }
    }
    
    fn show_placeholder(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        let placeholder = gtk::Label::builder()
            .label("No VPN profiles configured")
            .css_classes(["orbit-placeholder"])
            .margin_top(20)
            .build();
        self.list_box.append(&placeholder);
    }
    
    pub fn set_profiles(&self, profiles: Vec<VpnProfile>) {
        log::info!("VpnList: Received {} profiles", profiles.len());
        *self.profiles.borrow_mut() = profiles.clone();
        
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        if profiles.is_empty() {
            self.show_placeholder();
            return;
        }

        let section_header = gtk::Label::builder()
            .label("VPN CONNECTIONS")
            .css_classes(["orbit-section-header"])
            .halign(gtk::Align::Start)
            .build();
        self.list_box.append(&section_header);

        for profile in profiles {
            let row = self.create_vpn_row(&profile);
            self.list_box.append(&row);
        }
    }
    
    fn create_vpn_row(&self, profile: &VpnProfile) -> gtk::Box {
        let row = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .css_classes(["orbit-network-row"])
            .build();
        
        let icon_name = if profile.path == "external:riseup" {
            "network-vpn-symbolic" // Could use a custom riseup icon if available
        } else if profile.path == "external:tailscale" {
            "network-wireless-encrypted-symbolic"
        } else {
            "network-vpn-symbolic"
        };

        let icon = gtk::Image::builder()
            .icon_name(icon_name)
            .pixel_size(20)
            .css_classes(if profile.is_active { vec!["orbit-icon-accent"] } else { vec!["orbit-signal-icon"] })
            .valign(gtk::Align::Center)
            .build();
        row.append(&icon);
        
        let info_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .build();
        
        let name_label = if profile.is_external {
            format!("{} (App)", profile.name)
        } else {
            profile.name.clone()
        };

        let name = gtk::Label::builder()
            .label(&name_label)
            .css_classes(["orbit-ssid"])
            .halign(gtk::Align::Start)
            .build();
        info_box.append(&name);
        
        let status_text = if profile.is_active { "Connected" } else { &profile.vpn_type };
        let status = gtk::Label::builder()
            .label(status_text)
            .css_classes(["orbit-status"])
            .halign(gtk::Align::Start)
            .build();
        info_box.append(&status);
        
        row.append(&info_box);
        
        let toggle = gtk::Switch::builder()
            .active(profile.is_active)
            .css_classes(["orbit-toggle-switch"])
            .valign(gtk::Align::Center)
            .tooltip_text("Toggle VPN Connection")
            .build();
        
        let path = profile.path.clone();
        let on_toggle = self.on_toggle.clone();
        toggle.connect_state_set(move |_, state| {
            if let Some(cb) = on_toggle.borrow().as_ref() {
                cb(path.clone(), state);
            }
            gtk::glib::Propagation::Proceed
        });
        
        row.append(&toggle);
        row
    }
    
    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    pub fn set_on_toggle<F: Fn(String, bool) + 'static>(&self, callback: F) {
        *self.on_toggle.borrow_mut() = Some(Rc::new(callback));
    }
}
