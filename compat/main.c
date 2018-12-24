#include <bindings.h>

int main() {
    Alliance* alliance = Alliance_new_red(1);
    DriverStation* ds = DriverStation_new(4069, alliance);
    DriverStation_set_mode(ds, Teleoperated);
    DriverStation_enable(ds);
    for(;;) {
    }

    DriverStation_free(ds);
    return 0;
}
