CURRENT_TAG=$(git describe --tags)

sudo apt remove tirra
sudo dpkg -i packages/linux-release/tirra_$CURRENT_TAG.deb