# Binutils & GCC
AUTOCONF_VERSION=2.69
AUTOMAKE_VERSION=1.15.1
TOOLS_PATH=/opt/barbaos

wget https://ftp.gnu.org/gnu/automake/automake-$AUTOMAKE_VERSION.tar.gz
tar xf automake-$AUTOMAKE_VERSION.tar.gz
cd automake-$AUTOMAKE_VERSION
./configure --prefix=$TOOLS_PATH
make -j8
cd ..

wget https://ftp.gnu.org/gnu/autoconf/autoconf-$AUTOCONF_VERSION.tar.gz
tar xf autoconf-$AUTOCONF_VERSION.tar.gz
cd autoconf-$AUTOCONF_VERSION
configure --prefix=$TOOLS_PATH
make -j8
cd ..
