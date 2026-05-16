use anyhow::Result;
use std::path::Path;

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("hardware");
    std::fs::create_dir_all(&dest)?;

    let cmds = [
        ("lspci-nnvv.txt", "lspci -nnvv 2>/dev/null"),
        ("lsusb.txt", "lsusb 2>/dev/null"),
        ("dmidecode.txt", "dmidecode 2>/dev/null"),
        ("sensors.txt", "sensors -A 2>/dev/null"),
        ("cpu-info.txt", "cat /proc/cpuinfo 2>/dev/null"),
    ];

    for (filename, cmd) in &cmds {
        let output = crate::util::capture_cmd_output(cmd);
        std::fs::write(dest.join(filename), output)?;
    }

    // Kernel modules
    let modules = crate::util::capture_cmd_output("lsmod 2>/dev/null");
    std::fs::write(dest.join("lsmod.txt"), modules)?;

    Ok(())
}

pub fn validate() -> Result<()> {
    println!("  Post-apply system check:");

    // Check fans via sensors
    let sensors = crate::util::capture_cmd_output("sensors -A 2>/dev/null");
    let fan_lines: Vec<&str> = sensors.lines().filter(|l| l.contains("fan")).collect();
    if !fan_lines.is_empty() {
        println!("  Fan status:");
        for line in &fan_lines {
            println!("    {}", line.trim());
        }
    } else {
        println!("  Fan sensors: not detected or no fan entries");
    }

    // Check thermal
    let thermal_lines: Vec<&str> = sensors.lines().filter(|l| l.contains("temp")).collect();
    if !thermal_lines.is_empty() {
        println!("  Thermal:");
        for line in thermal_lines.iter().take(5) {
            println!("    {}", line.trim());
        }
    }

    // Check dmesg for firmware errors (last 10)
    let dmesg = crate::util::capture_cmd_output("dmesg -l err,warn 2>/dev/null | grep -i firmware | tail -5");
    if !dmesg.is_empty() {
        println!("  Firmware issues in dmesg:");
        for line in dmesg.lines() {
            println!("    {}", line);
        }
    } else {
        println!("  No firmware errors in dmesg ✓");
    }

    // GPU driver status
    let gpu = crate::util::capture_cmd_output("lspci -nnk 2>/dev/null | grep -A3 -E '(VGA|3D|Display)'");
    if !gpu.is_empty() {
        println!("  GPU:");
        for line in gpu.lines() {
            println!("    {}", line.trim());
        }
    }

    Ok(())
}
