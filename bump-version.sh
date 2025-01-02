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

cd backend
cargo set-version --bump patch
cd ../plugin
npm version patch
cd ..
cp plugin/manifest.json manifest.json  # for BRAT, otherwise it wouldn't update
git add .
TAG=$(node -p "require('./plugin/package.json').version")
git commit -m "Bump versions to $TAG"
echo "Tagging $TAG"
git tag -a $TAG -m "Release $TAG"
git push origin $TAG
echo "Done"
