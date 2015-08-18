#!/bin/sh

set -e
wget https://github.com/Feandil/kcov/archive/master.tar.gz
tar xzf master.tar.gz
mkdir kcov-master/build
cd kcov-master/build
cmake -DCMAKE_INSTALL_PREFIX=$HOME ..
make
make install
cd ../..

