#!/bin/zsh

# Get the current shell name
current_shell=$(echo $SHELL | awk -F '/' '{print $NF}')

# Initialize an empty variable for the RC file path
rc_file=""

# Determine the RC file based on the shell
case "$current_shell" in
  bash)
    rc_file="$HOME/.bashrc"
    ;;
  zsh)
    rc_file="$HOME/.zshrc"
    ;;
  # Add more shells and their RC files as needed
  *)
      echo "Unsupported shell: $current_shell"
      exit 1
      ;;
esac

# Check if the RC file exists
if [ -f "$rc_file" ]; then
  echo "Found RC file: $rc_file"
  # Example of sourcing the RC file
  # Note: Sourcing in a script like this will only affect the environment of the current script
  source "$rc_file"
else
  echo "RC file not found: $rc_file"
  exit 1
fi

index_array=(
  ./node_modules
  ./pnpm-lock.yaml
  ./website/{client,server}/node_modules
  ./website/{client,server}/pnpm-lock.yaml
  ./website/client/.nuxt
  ./website/server/dist
  ./packages/{common,node-native,wasm}/Cargo.lock
  ./packages/{common,node-native,wasm}/target
  ./packages/{common,node-native,wasm}/node_modules
  ./packages/wasm/dist
  ./packages/wasm/pkg
  ./packages/node-native/index.node
)

install_trash_cli() {
  prefix="[install_trash_cli] "
  if [[ -x "$(command -v trash)" ]]; then
    echo $prefix "trash-cli is installed"
  else
    echo $prefix "Creating temporary directory"

    mkdir -p /tmp/trash-cli
    cd /tmp/trash-cli

    echo $prefix "Cloning trash-cli"
    git clone https://github.com/ali-rantakari/trash.git trash && cd trash

    echo $prefix "Building trash-cli"
    make

    echo "$(ls)"
    echo ""
    echo $prefix "Copying trash-cli to /usr/local/bin"
    sudo cp trash /usr/local/bin

    echo $prefix "Building docs"
    make docs
    cp trash.1 /usr/local/share/man/man1/

    echo $prefix "Cleaning up"
    cd /tmp
    rm -rf /tmp/trash-cli
  fi
}

install_trash_cli_alias() {
  if [[ "$(command -v trm)" == "alias trm=/opt/homebrew/bin/trash" ]]; then
    echo "[install_trash_cli_alias] Alias for trm already exists"
  else
    prefix = "[install_trash_cli_alias] "

    echo $prefix "Creating alias for trm"

    echo $prefix "Adding alias to \"$rc_file\""
    echo "alias trm=trash" >> "$rc_file"

    source "$rc_file"
  fi

}

install_trash_cli;
install_trash_cli_alias;

# Initialize a counter for existing paths
existing_count=0

# Get the total number of paths
total_count=${#index_array[@]}

for path in "${index_array[@]}"
do
  if [ -e "$path" ]; then
    # Increment the counter for existing paths
    existing_count=$((existing_count + 1))

    echo "[$existing_count / $total_count] Moving $path to trash, because it exists"
    trm -rf $path
  else
    echo "Skipping $path, because it does not exist"
  fi
done
