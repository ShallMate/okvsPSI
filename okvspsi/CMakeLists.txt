
file(GLOB_RECURSE SRCS *.cpp)

add_executable(okvspsi  ${SRCS})

target_link_libraries(okvspsi volePSI_Tests)


if(VOLE_PSI_NO_LTO)
    target_link_options(okvspsi PUBLIC "-fno-lto")
endif()



if(MSVC)
    target_compile_options(okvspsi PRIVATE $<$<COMPILE_LANGUAGE:CXX>:/std:c++${VOLE_PSI_STD_VER}>)
    #target_compile_options(libOTe PRIVATE -openmp:experimental)
else()
    target_compile_options(okvspsi PRIVATE $<$<COMPILE_LANGUAGE:CXX>:-std=c++${VOLE_PSI_STD_VER}>)
endif()
