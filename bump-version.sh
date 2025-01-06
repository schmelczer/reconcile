#!/bin/bash

set -e

if [[ -z $1 ]]; then
  echo "Usage: $0 {patch|minor|major}"
  exit 1
fi

if [[ $1 =~ ^(patch|minor|major)$ ]]; then
  echo "Creating a new '$1' version"
else
  echo "Invalid argument: $1"
  echo "Usage: $0 {patch|minor|major}"
  exit 1
fi

if [[ -n $(git status --porcelain) ]]; then
  echo "Your working directory is not clean. Please commit or stash your changes before proceeding."
  exit 1
else
  echo "Your working directory is clean."
fi

echo "Bumping backend versions"
cd backend
cargo set-version --bump patch

echo "Bumping frontend versions"
cd ../plugin
npm version patch

echo "Updating frontend dependencies to match the new backend versions"
cd ../backend/sync_lib
wasm-pack build --target web --features console_error_panic_hook

cd ../../plugin
npm install

cd ..
cp plugin/manifest.json manifest.json  # for BRAT, otherwise it wouldn't update

# Commit and tag
git add .
TAG=$(node -p "require('./plugin/package.json').version")
git commit -m "Bump versions to $TAG"

git push
echo "Tagging $TAG"
git tag -a $TAG -m "Release $TAG"
git push origin $TAG
echo "Done"
