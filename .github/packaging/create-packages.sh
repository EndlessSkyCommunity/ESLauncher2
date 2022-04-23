#!/bin/bash
# Created by Sam Gleske (@samrocketman on GitHub)
# Sat 23 Apr 2022 11:48:30 AM EDT
# Ubuntu 20.04.4 LTS
# Linux 5.13.0-39-generic x86_64
# GNU bash, version 5.0.17(1)-release (x86_64-pc-linux-gnu)
# GNU Awk 5.0.1, API: 2.0 (GNU MPFR 4.0.2, GNU MP 6.2.0)
# DESCRIPTION
#     Create installable operating system packages for ESLauncher2.  This
#     provides a user friendly option to install ESLauncher2 for users who may
#     not be able to use the terminal.  This provides a standard way to install
#     ESLauncher2 software.  The final result being a DEB and RPM package.

set -eo pipefail

#############################
# PRE-FLIGHT ERROR CHECKING #
#############################

errors=()
if [ ! -d .git ]; then
  errors+=( 'ERROR: this script must be run from the root of the Git repository.' )
fi
if [ -z "${1:-}" ]; then
  errors+=( 'ERROR: First argument must be DEB/RPM package version of ESLauncher2.' )
fi
if [ ! -x target/release/eslauncher2 ] && [ ! -x eslauncher2-x86_64-unknown-linux-gnu ]; then
  errors+=( 'ERROR: ESLauncher does not appear to be built.' )
fi
if ! type -f fpm &> /dev/null; then
  errors+=( 'ERROR: fpm binary is missing.  Necessary for RPM and DEB packaging.' )
fi
if ! type -f rpmbuild &> /dev/null; then
  errors+=( 'ERROR: rpmbuild binary is missing.  Necessary for RPM packaging.' )
fi
# Exit non-zero if any errors found
if [ "${#errors[@]}" -gt 0 ]; then
  for x in "${errors[@]}"; do
    echo "$x" >&2
  done
  exit 1
fi

#############
# FUNCTIONS #
#############

# Gets a list of icons with sizes and prints their appropriate install path.
function get_icons() {
  \ls .github/packaging/icons/eslauncher2_[0-9]*x[0-9]*.png | awk -F_ '
    {
      file=$0;
      gsub(/^.*\//, "", $1);
      gsub(/\..*$/, "", $2);
    };
    {
      print file"=/usr/share/icons/hicolor/"$2"/apps/"$1".png";
    }'
}

#############
# VARIABLES #
#############

package_version="${1:-}"
# A bash array containing install paths for icons intended for FPM commands.
install_icons=( $(get_icons) )
binary=eslauncher2-x86_64-unknown-linux-gnu
git_long_commit="$(git rev-parse HEAD)"
description="ESLauncher2 manages Endless Sky installations.  Package built from https://github.com/EndlessSkyCommunity/ESLauncher2/tree/${git_long_commit}"

###############################
# CREATE DEB AND RPM PACKAGES #
###############################

# If local build set the appropriate bin path
if [ ! -x "${binary}" ]; then
  binary=target/release/eslauncher2
fi

for format in deb rpm; do
  (
    set -x
    fpm \
      -t "${format}" \
      -n eslauncher2 \
      --license gpl3 \
      --architecture x86_64 \
      --description "${description}" \
      -v "${package_version}" \
      -s dir \
      "${install_icons[@]}" \
      "./.github/packaging/eslauncher2.desktop=/usr/share/applications/eslauncher2.desktop" \
      "${binary}=/usr/bin/eslauncher2"
  )
done
