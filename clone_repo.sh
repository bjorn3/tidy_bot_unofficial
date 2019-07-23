#!/bin/bash

set -e

repo=$1
commit=$2

if [ ! -d rust ]; then
echo "===> Cloning https://github.com/rust-lang/rust.git"
git clone https://github.com/rust-lang/rust.git
fi

cd rust

repo_id=$(echo $commit | sha256sum - | tr -d "-")

echo "===> Cloning $repo"

if git remote | grep -q $repo_id; then
echo
else
git remote add $repo_id $repo
fi

git fetch $repo_id
git checkout $commit

echo "===> Checked out $commit"
