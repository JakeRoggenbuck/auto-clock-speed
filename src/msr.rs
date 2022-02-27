use core::arch::asm;

// MSR Dictionary
/// Platform Information, CPU Multiplier
pub const MSR_PLATFORM_INFO: u32 = 0xCE;

/// Overclocking / Undervolting Register
pub const MSR_OC_MAILBOX: u32 = 0x150;

/// Performance Status
pub const IA32_PERF_STATUS: u32 = 0x198;

/// Thermal Monitor Status (Read/Write)
pub const IA32_THERM_STATUS: u32 = 0x19C;

/// Temperature Target, Max Temperature Regulation
pub const MSR_TEMPERATURE_TARGET: u32 = 0x1A2;

/// Power Control Register
pub const MSR_POWER_CTL: u32 = 0x1FC;

/// Intel RAPL unit multipliers
pub const MSR_RAPL_POWER_UNIT: u32 = 0x606;

/// RAPL Package Power Limit Control (Read/Write)
pub const MSR_PKG_POWER_LIMIT: u32 = 0x610;

/// Current Turbo Power Limit
pub const MSR_TURBO_POWER_CURRENT_LIMIT: u32 = 0x1ac;

/// Intel CPU Package Energy Status
pub const MSR_INTEL_PKG_ENERGY_STATUS: u32 = 0x611;

/// Package RAPL Parameters (Read/Write)
pub const MSR_PKG_POWER_INFO: u32 = 0x614;

/// DRAM Energy Status
pub const MSR_DRAM_ENERGY_STATUS: u32 = 0x619;

/// PP0 RAPL Power Limit Control (Read/Write)
pub const MSR_PP0_POWER_LIMIT: u32 = 0x638;

/// PP0 Energy Status (Read Only)
pub const MSR_PP0_ENERGY_STATUS: u32 = 0x639;

/// PP0 Balance Policy (Read/Write)
pub const MSR_PP0_POLICY: u32 = 0x63a;

/// PP0 Performance Throttling Status (Read Only)
pub const MSR_PP0_PERF_STATUS: u32 = 0x63b;

/// PP1 (Usually GPU) Energy Status
pub const MSR_PP1_ENERGY_STATUS: u32 = 0x641;

/// DRAM Performance Throttling Status (Read Only)
pub const MSR_DRAM_PERF_STATUS: u32 = 0x61b;

/// Control TDP Limit (Read/Write)
pub const MSR_CONFIG_TDP_CONTROL: u32 = 0x64B;

/// Energy Performance Prefrence Control
pub const IA32_HWP_REQUEST: u32 = 0x774;

/*   CPU Power Measurement Interface Support Refrence Table  (PP0 usually cores, PP1 usually GPU)
Name 	                    Family Model   package PP0 PP1 DRAM PSys 	powercap 	perf_event 	PAPI
Sandybridge 	                6 	42 	        Y 	Y 	Y 	N 	N 	3.13 (2d281d8196) 	3.14 (4788e5b4b23) 	yes
Sandy Bridge EP 	            6 	45 	        Y 	Y 	N 	Y 	N 	3.13 (2d281d8196) 	3.14 (4788e5b4b23) 	yes
Ivy Bridge 	                    6 	58 	        Y 	Y 	Y 	N 	N 	3.13 (2d281d8196) 	3.14 (4788e5b4b23) 	yes
Ivy Bridge EP ("Ivy Town") 	    6 	62 	        Y 	Y 	N 	Y 	N 	no 	3.14 (4788e5b4b23) 	yes
Haswell 	                    6 	60 	        Y 	Y 	Y 	Y 	N 	3.16 (a97ac35b5d9) 	3.14 (4788e5b4b23) 	yes
Haswell ULT 	                6 	69 	        Y 	Y 	Y 	Y 	N 	3.13 (2d281d8196) 	3.14 (7fd565e27547) 	yes
Haswell GT3E 	                6 	70 	        Y 	Y 	Y 	Y 	N 	4.6 (462d8083f) 	4.6 (e1089602a3bf) 	yes
Haswell EP 	                    6 	63 	        Y 	? 	N 	Y 	N 	3.17 (64c7569c065) 	4.1 (64552396010) 	yes
Broadwell 	                    6 	61 	        Y 	Y 	Y 	Y 	N 	3.16 (a97ac35b5d9) 	4.1 (44b11fee517) 	yes
Broadwell-H GT3E 	            6 	71 	        Y 	Y 	Y 	Y 	N 	4.3 (4e0bec9e83) 	4.6 (7b0fd569303) 	yes
Broadwell-DE 	                6 	86 	        Y 	Y 	Y 	Y 	N 	3.19 (d72be771c5d) 	4.7 (31b84310c79) 	yes
Broadwell EP 	                6 	79 	        Y 	? 	N 	Y 	N 	4.1 (34dfa36c04c) 	4.6 (7b0fd569303) 	yes
Skylake Mobile 	                6 	78 	        Y 	Y 	Y 	Y 	Y 	4.1 (5fa0fa4b01) 	4.7 (dcee75b3b7f02) 	yes
Skylake Desktop H/S 	        6 	94 	        Y 	Y 	Y 	Y 	Y 	4.3 (2cac1f70) 	4.7 (dcee75b3b7f02) 	yes
Skylake Server 	                6 	85 	        Y 	Y 	N 	Y 	N 	4.8??? 	4.8 (348c5ac6c7dc11) 	yes
Kabylake 	                    6 	142,158 	Y 	Y 	Y 	Y 	Y 	4.7 (6c51cc0203) 	4.11 (f2029b1e47) 	yes
Cannonlake 	                    6 	102 	    Y 	Y 	Y 	Y 	Y 	4.17 (?) 	4.17 (490d03e83da2) 	?
Knights Landing 	            6 	87 	        Y 	N 	N 	Y 	N 	4.2 (6f066d4d2) 	4.6 (4d120c535d6) 	yes
Knights Mill 	                6 	133 	    Y 	N 	N 	Y 	N 	() 	4.9 (36c4b6c14d20) 	yes
Atom Goldmont 	                6 	92 	        Y 	Y 	Y 	Y 	N 	4.4 (89e7b2553a) 	4.9 (2668c6195685) 	yes
Atom Denverton 	                6 	95 	        Y 	Y 	Y 	Y 	N 	() 	4.14 (450a97893559354) 	yes
Atom Gemini Lake 	            6 	122 	    Y 	Y 	Y 	Y 	N 	() 	4.14 (450a97893559354) 	yes
Atom Airmont / Braswell 	    6 	76 	        ? 	? 	? 	? 	N 	3.19 (74af752e4895) 	no 	no
Atom Tangier / Merrifield 	    6 	74 	        ? 	? 	? 	? 	N 	3.19 (74af752e4895) 	no 	no
Atom Moorefield / Annidale 	    6 	90 	        ? 	? 	? 	? 	N 	3.19 (74af752e4895) 	no 	no
Atom Silvermont / Valleyview 	6 	55 	        ? 	? 	? 	? 	N 	3.13 (ed93b71492d) 	no 	no
 */

/// Reads u64 from MSR
///
/// CPL 0 Required
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[allow(unused_mut)]
pub unsafe fn msr_read(reg: u32) -> u64 {
    let (high, low): (u32, u32);
    asm!("msr_read",
    out("eax") low,
    out("edx") high,
    in("ecx") reg);
    ((high as u64) << 32) | (low as u64)
}
