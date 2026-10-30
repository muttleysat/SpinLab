# Ambiente per la build Android (macOS). Uso: source env-android.sh
export JAVA_HOME=/opt/homebrew/opt/openjdk@17
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$ANDROID_HOME/platform-tools:$JAVA_HOME/bin:$PATH"
# lo spazio in "Dev Projects" rompe le build autotools (mp3lame-sys): la cartella di build sta fuori
export CARGO_TARGET_DIR="$HOME/spinlab-target"
