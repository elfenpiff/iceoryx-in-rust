use std::sync::atomic::{AtomicU64, Ordering};

use elkodon_bb_elementary::scope_guard::*;
use elkodon_bb_testing::assert_that;

#[test]
fn scope_guard_callbacks_are_called_correctly_success_case() {
    let startup_callback = AtomicU64::new(0);
    let cleanup_callback = AtomicU64::new(0);

    {
        let guard = ScopeGuardBuilder::new(456)
            .on_init(|v| -> Result<(), ()> {
                startup_callback.store(*v, Ordering::Relaxed);
                Ok(())
            })
            .on_drop(|v| {
                cleanup_callback.store(*v, Ordering::Relaxed);
            })
            .create();

        assert_that!(guard, is_ok);

        assert_that!(startup_callback.load(Ordering::Relaxed), eq 456);
        assert_that!(cleanup_callback.load(Ordering::Relaxed), eq 0);

        let mut guard = guard.unwrap();
        *guard.get_mut() = 991;

        startup_callback.store(0, Ordering::Relaxed);
    }

    assert_that!(startup_callback.load(Ordering::Relaxed), eq 0);
    assert_that!(cleanup_callback.load(Ordering::Relaxed), eq 991);
}

#[test]
fn scope_guard_callbacks_are_called_correctly_failure_case() {
    let startup_callback = AtomicU64::new(0);
    let cleanup_callback = AtomicU64::new(0);

    {
        let guard = ScopeGuardBuilder::new(789)
            .on_init(|v| -> Result<(), u64> {
                startup_callback.store(*v, Ordering::Relaxed);
                Err(23482)
            })
            .on_drop(|v| {
                cleanup_callback.store(*v, Ordering::Relaxed);
            })
            .create();

        assert_that!(guard, is_err);
        assert_that!(guard.err().unwrap(), eq 23482);
        assert_that!(startup_callback.load(Ordering::Relaxed), eq 789);
        assert_that!(cleanup_callback.load(Ordering::Relaxed), eq 0);

        startup_callback.store(0, Ordering::Relaxed);
    }

    assert_that!(startup_callback.load(Ordering::Relaxed), eq 0);
    assert_that!(cleanup_callback.load(Ordering::Relaxed), eq 0);
}
