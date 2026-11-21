//! Invitations on the phone: hands the plan from `kuracare_core::reminders` to
//! the Kotlin side (`android/kotlin/org/kuracare/app/Bridge.kt`) over JNI.
//! Everywhere else (desktop, tests) these calls do nothing.
//!
//! Every call fails quietly: if notifications cannot work (for example in a
//! debug APK built without our Kotlin code), the app works as before.

/// The Kotlin side found an invitation tapped by the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tapped {
    pub kind: String,
    pub key: String,
}

impl Tapped {
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    fn parse(s: &str) -> Option<Self> {
        let (kind, key) = s.split_once('|')?;
        Some(Self {
            kind: kind.into(),
            key: key.into(),
        })
    }

    pub fn is_test(&self) -> bool {
        self.key.starts_with("test")
    }
}

pub use imp::*;

#[cfg(target_os = "android")]
mod imp {
    use jni::objects::{JClass, JObject, JString, JValue};
    use jni::{JNIEnv, JavaVM};

    use super::Tapped;

    const BRIDGE: &str = "org.kuracare.app.Bridge";
    const CTX: &str = "Landroid/content/Context;";

    /// Runs `f` with the JNI env, the Android context and the Bridge class.
    fn with_bridge<R>(
        f: impl for<'a> FnOnce(&mut JNIEnv<'a>, &JObject<'a>, &JClass<'a>) -> jni::errors::Result<R>,
    ) -> Option<R> {
        let ctx = ndk_context::android_context();
        // SAFETY: both pointers come from ndk-context, set by tao at startup and
        // valid for the life of the process.
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
        let mut env = vm.attach_current_thread().ok()?;
        let res = env.with_local_frame(16, |env| {
            let context = unsafe { JObject::from_raw(ctx.context().cast()) };
            // Load through the app's class loader: FindClass from a native
            // thread only sees system classes.
            let loader = env
                .call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
                .l()?;
            let name = env.new_string(BRIDGE)?;
            let class = env
                .call_method(
                    &loader,
                    "loadClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[JValue::Object(&name)],
                )?
                .l()?;
            let class = JClass::from(class);
            f(env, &context, &class)
        });
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_describe();
            let _ = env.exception_clear();
        }
        match res {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("kuracare: notifications unavailable ({e})");
                None
            }
        }
    }

    fn call_with_string(method: &str, arg: &str) {
        with_bridge(|env, ctx, class| {
            let s = env.new_string(arg)?;
            env.call_static_method(
                class,
                method,
                format!("({CTX}Ljava/lang/String;)V"),
                &[JValue::Object(ctx), JValue::Object(&s)],
            )?;
            Ok(())
        });
    }

    fn call_with_context(method: &str) {
        with_bridge(|env, ctx, class| {
            env.call_static_method(class, method, format!("({CTX})V"), &[JValue::Object(ctx)])?;
            Ok(())
        });
    }

    /// Replaces the plan (JSON from `Plan::to_json`) and arms the next alarm.
    pub fn set_plan(json: &str) {
        call_with_string("setPlan", json);
    }

    /// Shows the first invitation of `json` (plan format) right away.
    pub fn show_now(json: &str) {
        call_with_string("showNow", json);
    }

    pub fn allowed() -> bool {
        with_bridge(|env, ctx, class| {
            env.call_static_method(
                class,
                "notificationsAllowed",
                format!("({CTX})Z"),
                &[JValue::Object(ctx)],
            )?
            .z()
        })
        .unwrap_or(false)
    }

    /// Android's own permission dialog (once; afterwards see `open_settings`).
    pub fn ask() {
        call_with_context("askNotifications");
    }

    pub fn open_settings() {
        call_with_context("openNotificationSettings");
    }

    pub fn take_tapped() -> Option<Tapped> {
        with_bridge(|env, _ctx, class| {
            let v = env
                .call_static_method(class, "takeInvite", "()Ljava/lang/String;", &[])?
                .l()?;
            if v.is_null() {
                return Ok(None);
            }
            let s: String = env.get_string(&JString::from(v))?.into();
            Ok(Tapped::parse(&s))
        })
        .flatten()
    }

    /// True where invitations can work at all.
    pub fn supported() -> bool {
        true
    }

    /// Colours of the status and navigation bars (ARGB); mode 0 auto, 1 light, 2 dark.
    pub fn set_bars(day: i32, night: i32, mode: i32) {
        with_bridge(|env, ctx, class| {
            env.call_static_method(
                class,
                "setBars",
                format!("({CTX}III)V"),
                &[
                    JValue::Object(ctx),
                    JValue::Int(day),
                    JValue::Int(night),
                    JValue::Int(mode),
                ],
            )?;
            Ok(())
        });
    }
}

#[cfg(not(target_os = "android"))]
mod imp {
    use super::Tapped;

    pub fn set_plan(_json: &str) {}
    pub fn show_now(_json: &str) {}
    pub fn allowed() -> bool {
        false
    }
    pub fn ask() {}
    pub fn open_settings() {}
    pub fn take_tapped() -> Option<Tapped> {
        None
    }
    pub fn supported() -> bool {
        false
    }
    pub fn set_bars(_day: i32, _night: i32, _mode: i32) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_what_kotlin_hands_over() {
        let t = Tapped::parse("checkin|checkin:2026-11-21:0").unwrap();
        assert_eq!(t.kind, "checkin");
        assert_eq!(t.key, "checkin:2026-11-21:0");
        assert!(!t.is_test());
        assert!(Tapped::parse("test|test").unwrap().is_test());
        assert!(Tapped::parse("nonsense").is_none());
    }
}
