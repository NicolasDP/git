#! /bin/bash

[ "${TRAVIS_BRANCH}" = "master" ] || exit 0
[ "${TRAVIS_PULL_REQUEST}" = "false" ] || exit 0

cargo doc

echo "<meta http-equiv=refresh content=0;url=git/index.html>" > target/doc/index.html
sudo pip install ghp-import
ghp-import -n target/doc
git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
