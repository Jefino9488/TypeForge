cmake_minimum_required(VERSION 3.10)
project(fcitx5-typeforge VERSION 0.1.0)

find_package(ECM 1.0.0 REQUIRED)
set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} ${CMAKE_MODULE_PATH})

find_package(Fcitx5Core 5.0.0 REQUIRED)
find_package(Fcitx5Utils REQUIRED)
find_package(Gettext REQUIRED)

# The rust library we are going to link against
# It will be built in the target directory by Cargo
set(RUST_BRIDGE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../target/release")
set(RUST_BRIDGE_LIB "${RUST_BRIDGE_DIR}/libtypeforge_fcitx5_bridge.a")

add_library(fcitx5-typeforge MODULE
    TypeForgeEngine.cpp
)

target_link_libraries(fcitx5-typeforge
    Fcitx5::Core
    Fcitx5::Utils
    -Wl,--whole-archive ${RUST_BRIDGE_LIB} -Wl,--no-whole-archive
    dl pthread m
)

set_target_properties(fcitx5-typeforge PROPERTIES PREFIX "")

configure_file(typeforge.conf.in typeforge.conf @ONLY)
fcitx5_translate_desktop_file(typeforge-im.conf.in typeforge-im.conf)

install(TARGETS fcitx5-typeforge DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/typeforge.conf" DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon")
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/typeforge-im.conf" DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod")
