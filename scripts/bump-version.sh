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

echo "Bumping versions"
cargo set-version --bump $1

wasm-pack build --target web --features wasm,wee_alloc

cd reconcile-js
npm version $1

cd -

git add .
TAG=$(node -p "require('./reconcile-js/package.json').version")

git commit -m "Bump versions to $TAG"

git push
echo "Tagging $TAG"
git tag -a $TAG -m "Release $TAG"
git push origin $TAG
echo "Done"
