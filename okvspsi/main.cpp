#include "tests/UnitTests.h"
#include "perf.h"

#include "volePSI/fileBased.h"

int main(int argc, char** argv)
{
    oc::CLP cmd(argc, argv);

    //perfOkvsPSI(cmd);
    networkSocketExampleRun(cmd);
    return 0;
}