// SPDX-License-Identifier: GPL-2.0

//! Rust Asix PHYs driver
use kernel::c_str;
use kernel::net::phy;
use kernel::prelude::*;

module! {
    type: RustAsixPhy,
    name: "rust_asix_phy",
    author: "Rust for Linux Contributors",
    description: "Rust Asix PHYs driver",
    license: "GPL",
}

const MII_BMCR: u32 = 0x00;
const BMCR_SPEED100: u32 = 0x2000;
const BMCR_FULLDPLX: u32 = 0x0100;

struct PhyAx88772A {}

impl PhyAx88772A {
    const PHY_ID_ASIX_AX88772A: u32 = 0x003b1861;
}

#[vtable]
impl phy::Driver for PhyAx88772A {
    const FLAGS: u32 = phy::PHY_IS_INTERNAL;

    fn match_phy_device(dev: &mut phy::Device) -> bool {
        dev.id() == Self::PHY_ID_ASIX_AX88772A
    }

    fn read_status(dev: &mut phy::Device) -> Result {
        dev.update_link()?;
        if !dev.link() {
            return Ok(());
        }
        let ret = dev.read(MII_BMCR)?;

        if ret as u32 & BMCR_SPEED100 != 0 {
            dev.set_speed(100);
        } else {
            dev.set_speed(10);
        }

        let duplex = ret as u32 & BMCR_FULLDPLX != 0;
        dev.set_duplex(duplex);

        dev.read_lpa()?;

        if dev.is_autoneg_enabled() && dev.is_autoneg_completed() {
            dev.resolve_aneg_linkmode();
        }

        Ok(())
    }

    fn suspend(dev: &mut phy::Device) -> Result {
        dev.suspend()
    }

    fn resume(dev: &mut phy::Device) -> Result {
        dev.resume()
    }

    fn soft_reset(dev: &mut phy::Device) -> Result {
        dev.write(MII_BMCR, 0)?;
        dev.soft_reset()
    }

    fn link_change_notify(dev: &mut phy::Device) {
        if dev.state() == phy::DeviceState::NoLink {
            let _ = dev.init_hw();
            let _ = dev.start_aneg();
        }
    }
}

struct RustAsixPhy {
    _reg: phy::Registration<1>,
}

impl kernel::Module for RustAsixPhy {
    fn init(module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust Asix phy driver\n");
        let mut reg: phy::Registration<1> = phy::Registration::new(module);

        reg.register(&phy::Adapter::<PhyAx88772A>::new(c_str!(
            "Asix Electronics AX88772A"
        )))?;
        Ok(RustAsixPhy { _reg: reg })
    }
}

impl Drop for RustAsixPhy {
    fn drop(&mut self) {
        pr_info!("Rust Asix phy driver (exit)\n");
    }
}

kernel::phy_module_device_table!(
    __mod_mdio__asix_tbl_device_table,
    [(PhyAx88772A::PHY_ID_ASIX_AX88772A, 0xffffffff)]
);
