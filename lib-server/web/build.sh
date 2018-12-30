# check to see if pre-built by an earlier stage
which yarn
if [ $? != 0 ] ; then
  if [ -f dist/css/weaver.css ] ; then
    exit 0
  fi
fi

# required file does not exist, try to build. This will require yarn

yarn 
yarn run build
