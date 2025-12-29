#!/bin/bash

log() {
    local level="$1"
    shift
    local message="$*"

    case "$level" in
        trace)
            if [ "$is_verbose" = true ]; then
                echo "[TRACE] $message"
            fi
            ;;
        info)
            echo "[INFO] $message"
            ;;
        error)
            echo "[ERROR] $message" >&2
            ;;
        *)
            echo "[UNKNOWN] $message"
            ;;
    esac
}

count=0
for arg in "$@"; do
    case "$arg" in
        install|upgrade|uninstall)
            action="$arg"
            ((count++))
        ;;
    esac
done

if (( count != 1 )); then
    log error "Specify exactly one of: install, upgrade, uninstall."
    exit 1
fi

is_verbose=false
for arg in "$@"; do
    case "$arg" in
        --verbose|-v)
            is_verbose=true
        ;;
    esac
done

task_bin="$HOME/.task/bin"

if [ "$action" = "install" ]; then
    for cmd in curl jq; do
        if ! command -v "$cmd" &> /dev/null; then
            log error "$cmd is required but not installed"
            exit 1
        fi
    done

    if [ -f "$task_bin/task" ]; then
        log info "Task binary already exists at $task_bin/task. Use 'upgrade' to update it instead of 'install'."
        exit 0
    fi

    if [ -d "$HOME/.task" ]; then
        log trace "$HOME/.task already exists, skipping"
        [ ! -d "$task_bin" ] && mkdir "$task_bin" && log trace "Created $task_bin folder"
    else
        log trace "Creating $HOME/.task and $task_bin folders"
        mkdir -p "$task_bin"
    fi
    log info "$HOME/.task initialized"

    os=$(uname -s)
    case "$os" in
        Darwin) os="macos"; log trace "OS detected: macOS" ;;
        Linux)  os="linux"; log trace "OS detected: Linux" ;;
        *) log error "Unknown OS: $os"; exit 1 ;;
    esac

    arch=$(uname -m)
    case "$arch" in
        x86_64) arch="x86_64"; log trace "Architecture detected: x86_64 (64-bit Intel/AMD)" ;;
        arm64|aarch64) arch="arm64"; log trace "Architecture detected: ARM64" ;;
        *) log error "Unknown architecture: $arch"; exit 1 ;;
    esac

    binary_name="task-$os-$arch"
    repo="thespragg/task"

    log trace "Fetching latest release tag for $repo"
    latest_tag=$(curl -s "https://api.github.com/repos/$repo/releases/latest" | jq -r .tag_name)

    if [ -z "$latest_tag" ] || [ "$latest_tag" = "null" ]; then
        log error "Failed to fetch latest release tag"
        exit 1
    fi

    log trace "Fetching asset URL for $binary_name"
    asset_url=$(curl -s "https://api.github.com/repos/$repo/releases/latest" \
        | jq -r ".assets[] | select(.name==\"$binary_name\") | .browser_download_url")

    if [ -z "$asset_url" ] || [ "$asset_url" = "null" ]; then
        log error "No binary found for $binary_name"
        exit 1
    fi

    log info "Downloading $binary_name..."

    if curl -L -o "$binary_name" "$asset_url" >/dev/null 2>&1; then
        mv "$binary_name" "$task_bin"
        mv "$task_bin/$binary_name" "$task_bin/task"
        chmod +x "$task_bin/task"
        log info "Downloaded and moved binary to $task_bin"
    else
        log error "Failed to download $binary_name from $asset_url"
        exit 1
    fi

    user_shell=$(basename "$SHELL") 

    add_to_path() {
        local shell_profile="$1"
        local path_entry="export PATH=\"$task_bin:\$PATH\""

        if ! grep -Fxq "$path_entry" "$shell_profile"; then
            echo "$path_entry" >> "$shell_profile"
            log info "Added $task_bin to PATH in $shell_profile"
        else
            log trace "$task_bin already in PATH in $shell_profile"
        fi
    }

    case "$user_shell" in
        bash) profile="$HOME/.bashrc" ;;
        zsh) profile="$HOME/.zshrc" ;;
        fish) profile="$HOME/.config/fish/config.fish" ;;
        *) log info "Unsupported shell: $user_shell. Please add $task_bin to PATH manually."; exit 0 ;;
    esac

    [ ! -f "$profile" ] && touch "$profile"
    add_to_path "$profile"

    export PATH="$task_bin:$PATH"
    log trace "Updated current PATH for this session"

    log info "Installing worker service..."
    task_exe="$task_bin/task"
    
    case "$os" in
        macos)
            plist_dir="$HOME/Library/LaunchAgents"
            plist_file="$plist_dir/com.user.taskworker.plist"
            
            [ ! -d "$plist_dir" ] && mkdir -p "$plist_dir"
            
            cat > "$plist_file" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>com.user.taskworker</string>
    <key>ProgramArguments</key>
    <array>
      <string>$task_exe</string>
      <string>--worker</string>
      <string>--folder</string>
      <string>$HOME/.task</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/$HOME/.task/logs/task.out.log</string>
    <key>StandardErrorPath</key>
    <string>/$HOME/.task/logs/task.err.log</string>
  </dict>
</plist>
EOF
            
            log trace "Created launchd plist at $plist_file"
            
            if launchctl load "$plist_file" 2>/dev/null; then
                log info "Loaded launchd service"
            else
                log trace "launchd service will be loaded on next login"
            fi
            ;;
            
        linux)
            systemd_dir="$HOME/.config/systemd/user"
            service_file="$systemd_dir/task-worker.service"
            
            [ ! -d "$systemd_dir" ] && mkdir -p "$systemd_dir"
            
            cat > "$service_file" <<EOF
[Unit]
Description=Task CLI Worker
After=network.target

[Service]
ExecStart=$task_exe --worker --folder $HOME/.task
Restart=always
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=default.target
EOF
            
            log trace "Created systemd service at $service_file"
            
            if systemctl --user daemon-reload 2>/dev/null; then
                log trace "Reloaded systemd user daemon"
                
                if systemctl --user enable task-worker 2>/dev/null; then
                    log trace "Enabled task-worker service"
                fi
                
                if systemctl --user start task-worker 2>/dev/null; then
                    log info "Started task-worker service"
                else
                    log trace "task-worker service will start on next login"
                fi
            else
                log trace "systemd user services will be available after login"
            fi
            ;;
    esac

    log info "Installation complete!"
    echo ""

    if "$task_bin/task" --version 2>/dev/null; then
        echo ""
    else
        log trace "Could not retrieve version"
    fi

    echo "To use task, either:"
    echo "  1. Restart your terminal, or"
    echo "  2. Run: source $profile"
    echo ""
    echo "Then verify with: task --version"
    echo ""
    echo "Worker service installed and will run automatically."

elif [ "$action" = "upgrade" ]; then
    log info "Upgrading task..."
    
    if [ ! -f "$task_bin/task" ]; then
        log error "task is not installed. Run 'install' first."
        exit 1
    fi
    
    for cmd in curl jq; do
        if ! command -v "$cmd" &> /dev/null; then
            log error "$cmd is required but not installed"
            exit 1
        fi
    done
    
    old_version=$("$task_bin/task" --version 2>/dev/null || echo "unknown")
    log trace "Current version: $old_version"

    os=$(uname -s)
    case "$os" in
        Darwin) os="macos"; log trace "OS detected: macOS" ;;
        Linux)  os="linux"; log trace "OS detected: Linux" ;;
        *) log error "Unknown OS: $os"; exit 1 ;;
    esac

    arch=$(uname -m)
    case "$arch" in
        x86_64) arch="x86_64"; log trace "Architecture detected: x86_64" ;;
        arm64|aarch64) arch="arm64"; log trace "Architecture detected: ARM64" ;;
        *) log error "Unknown architecture: $arch"; exit 1 ;;
    esac

    binary_name="task-$os-$arch"
    repo="thespragg/task"

    log trace "Fetching latest release tag for $repo"
    latest_tag=$(curl -s "https://api.github.com/repos/$repo/releases/latest" | jq -r .tag_name)

    if [ -z "$latest_tag" ] || [ "$latest_tag" = "null" ]; then
        log error "Failed to fetch latest release tag"
        exit 1
    fi

    log trace "Fetching asset URL for $binary_name"
    asset_url=$(curl -s "https://api.github.com/repos/$repo/releases/latest" \
        | jq -r ".assets[] | select(.name==\"$binary_name\") | .browser_download_url")

    if [ -z "$asset_url" ] || [ "$asset_url" = "null" ]; then
        log error "No binary found for $binary_name"
        exit 1
    fi

    log info "Downloading latest version of $binary_name..."

    if curl -L -o "$binary_name" "$asset_url" >/dev/null 2>&1; then
        mv "$binary_name" "$task_bin/task"
        chmod +x "$task_bin/task"
        log info "Upgraded binary at $task_bin/task"
    else
        log error "Failed to download $binary_name from $asset_url"
        exit 1
    fi
    
    case "$os" in
        macos)
            plist_file="$HOME/Library/LaunchAgents/com.user.taskworker.plist"
            if [ -f "$plist_file" ]; then
                log info "Restarting launchd service..."
                launchctl unload "$plist_file" 2>/dev/null || true
                launchctl load "$plist_file" 2>/dev/null || true
                log trace "Restarted launchd service"
            fi
            ;;
        linux)
            service_file="$HOME/.config/systemd/user/task-worker.service"
            if [ -f "$service_file" ]; then
                log info "Restarting systemd service..."
                systemctl --user restart task-worker 2>/dev/null || true
                log trace "Restarted systemd service"
            fi
            ;;
    esac

    new_version=$("$task_bin/task" --version 2>/dev/null || echo "unknown")
    log info "Upgraded from: $old_version, to: $new_version."
    echo ""
    echo "Run 'task --version' to verify"

elif [ "$action" = "uninstall" ]; then
    log info "Uninstalling task..."
    
    os=$(uname -s)
    case "$os" in
        Darwin) os="macos" ;;
        Linux)  os="linux" ;;
        *) log error "Unknown OS: $os"; exit 1 ;;
    esac
    
    case "$os" in
        macos)
            plist_file="$HOME/Library/LaunchAgents/com.user.taskworker.plist"
            if [ -f "$plist_file" ]; then
                log info "Stopping and removing launchd service..."
                launchctl unload "$plist_file" 2>/dev/null || true
                rm "$plist_file"
                log trace "Removed launchd service"
            fi
            ;;
        linux)
            service_file="$HOME/.config/systemd/user/task-worker.service"
            if [ -f "$service_file" ]; then
                log info "Stopping and removing systemd service..."
                systemctl --user stop task-worker 2>/dev/null || true
                systemctl --user disable task-worker 2>/dev/null || true
                rm "$service_file"
                systemctl --user daemon-reload 2>/dev/null || true
                log trace "Removed systemd service"
            fi
            ;;
    esac
    
    if [ -d "$HOME/.task" ]; then
        rm -rf "$HOME/.task"
        log info "Removed $HOME/.task directory"
    fi
    
    user_shell=$(basename "$SHELL")
    case "$user_shell" in
        bash) profile="$HOME/.bashrc" ;;
        zsh) profile="$HOME/.zshrc" ;;
        fish) profile="$HOME/.config/fish/config.fish" ;;
        *) profile="" ;;
    esac
    
    if [ -n "$profile" ] && [ -f "$profile" ]; then
        if grep -q "export PATH=\"$task_bin:\$PATH\"" "$profile"; then
            sed -i.bak "/export PATH=\"${task_bin//\//\\/}:\$PATH\"/d" "$profile"
            rm -f "$profile.bak"
            log info "Removed PATH entry from $profile"
        fi
    fi

    log info "Uninstall complete!"
    echo ""
    echo "Please restart your terminal or run: source $profile"
fi