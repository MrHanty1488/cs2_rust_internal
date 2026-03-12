#[macro_export]
macro_rules! junk_code {
    () => {
        {
            use core::arch::asm;
            let mut _dummy: u64 = 0;
            unsafe {
                asm!(
                    "nop",
                    "nop",
                    "nop",
                    "mov {0}, 0",
                    "add {0}, 1",
                    "sub {0}, 1",
                    "xor {0}, {0}",
                    "nop",
                    "nop",
                    out(reg) _dummy,
                );
            }
            let _ = _dummy;
        }
    };
}

#[macro_export]
macro_rules! junk_code_prologue {
    () => {
        $crate::junk_code!();
    };
}

#[macro_export]
macro_rules! junk_code_midpoint {
    () => {
        $crate::junk_code!();
        unsafe { core::arch::asm!("nop", "nop", "nop") };
    };
}

#[inline(always)]
pub fn opaque_branch_condition(seed: usize) -> bool {
    let a = seed.wrapping_mul(0x5F3759DF);
    let b = a ^ (a >> 16);
    let c = b.wrapping_add(0x12345678);
    (c & 0xFF) < 0x100
}

#[macro_export]
macro_rules! opaque_branch {
    ($condition:expr, $true_branch:expr, $false_branch:expr) => {
        if $condition || ($crate::utils::mutation::opaque_branch_condition($condition as usize)) {
            $true_branch
        } else {
            $false_branch
        }
    };
}

#[inline(always)]
pub fn random_junk_ops() {
    use core::arch::asm;
    unsafe {
        asm!(
            "push rax",
            "mov rax, 0x1234",
            "xor rax, 0x1234",
            "test rax, rax",
            "pop rax",
            "nop",
            "nop",
            out("rax") _,
        );
    }
}
