# Distributed under the OSI-approved BSD 3-Clause License.  See accompanying
# file LICENSE.rst or https://cmake.org/licensing for details.

cmake_minimum_required(VERSION ${CMAKE_VERSION}) # this file comes with cmake

# If CMAKE_DISABLE_SOURCE_CHANGES is set to true and the source directory is an
# existing directory in our source tree, calling file(MAKE_DIRECTORY) on it
# would cause a fatal error, even though it would be a no-op.
if(NOT EXISTS "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-src")
  file(MAKE_DIRECTORY "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-src")
endif()
file(MAKE_DIRECTORY
  "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-build"
  "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix"
  "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix/tmp"
  "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix/src/json-populate-stamp"
  "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix/src"
  "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix/src/json-populate-stamp"
)

set(configSubDirs )
foreach(subDir IN LISTS configSubDirs)
    file(MAKE_DIRECTORY "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix/src/json-populate-stamp/${subDir}")
endforeach()
if(cfgdir)
  file(MAKE_DIRECTORY "/Users/adafaralph/dev/Beacon2.0/build/_deps/json-subbuild/json-populate-prefix/src/json-populate-stamp${cfgdir}") # cfgdir has leading slash
endif()
