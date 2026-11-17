# Running violence on an Android phone

violence is a [Dioxus 0.7](https://dioxuslabs.com/learn/0.7) app built for Android.
Alarm scheduling only exists on Android, so a phone (or emulator) is the way to
test it properly.

## 1. Prerequisites

| Tool | Version used by CI | Notes |
|---|---|---|
| Rust (stable) | latest | plus the Android targets below |
| Dioxus CLI (`dx`) | `0.7.5` | should match the `dioxus` version in `Cargo.toml` |
| JDK | 17 or 21 | used by Gradle |
| Android SDK | platform-tools, `platforms;android-34`, `build-tools;34.0.0` | provides `adb` |
| Android NDK | `27.0.12077973` | |

Install the Rust targets and the CLI:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
cargo install dioxus-cli --version 0.7.5 --locked
```

Point the build at your SDK, NDK and JDK (adjust the paths to your install),
for example in your shell profile:

```sh
export ANDROID_HOME=/opt/android-sdk
export ANDROID_NDK_HOME=/opt/android-ndk
export JAVA_HOME=/usr/lib/jvm/java-17-openjdk
```

Optional – custom launcher icon. Dioxus does not apply our icon yet
([DioxusLabs/dioxus#3685](https://github.com/DioxusLabs/dioxus/issues/3685)), so
install the Gradle overlay once:

```sh
mkdir -p ~/.gradle/init.d
ln -sf "$(pwd)/android/icon-overlay.gradle" ~/.gradle/init.d/violence-icon-overlay.gradle
```

## 2. Connect the phone

1. On the phone, enable **Developer options** (Settings → About phone → tap
   *Build number* seven times).
2. In **Developer options**, turn on **USB debugging**.
3. Plug the phone in and accept the *Allow USB debugging?* prompt.
4. Check that it shows up as `device` (not `unauthorized`):

   ```sh
   adb devices -l
   ```

Wireless debugging (`adb pair` / `adb connect`) works too, as long as the phone
is listed by `adb devices`.

## 3. Run the app (development)

From the repository root:

```sh
dx serve --platform android
```

`dx` builds the app, regenerates `assets/tailwind.css` from `tailwind.css`,
installs the APK on the phone and launches it. It keeps running and rebuilds
when you edit files. Useful keys while it runs: `r` rebuild, `v` verbose logs,
`Ctrl+C` stop.

With more than one device connected, pick one by its serial from
`adb devices`:

```sh
dx serve --platform android --device <serial>
```

To see the app's logs:

```sh
adb logcat --pid=$(adb shell pidof com.example.violence)
```

## 4. Build a release APK

This is what CI runs:

```sh
dx bundle --platform android --release --target aarch64-linux-android
```

The APK ends up under `target/dx/…/release/`. To install it on the phone you
need to sign it (see `.github/workflows/android.yml` for the `zipalign` +
`apksigner` steps), then:

```sh
adb install -r path/to/violence.apk
```

## Troubleshooting

- **`dx and dioxus versions are incompatible`** – your `dx` differs from the
  `dioxus` crate version. The app usually still runs, but hot reload may
  misbehave. Install the matching CLI:
  `cargo install dioxus-cli --version <version> --locked`.
- **Phone missing or `unauthorized` in `adb devices`** – re-plug the cable,
  accept the debugging prompt, or run `adb kill-server && adb devices`.
- **`Waiting for android emulator to be ready...`** – this message also shows
  for physical phones; the app launches shortly after.
- **Building for desktop fails on `openssl-sys`** – the default `mobile`
  feature is meant for Android. Use the phone; the unit tests still run on the
  host with `cargo test --no-default-features`.
