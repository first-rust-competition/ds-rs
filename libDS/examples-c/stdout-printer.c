#include <libDS.h>
#include <stdio.h>

void callback(StdoutMessage msg) {
    printf("Got message %s\n", msg.message);
}

int main(void) {
    Alliance* alliance = DS_Alliance_new_red(1);
    DriverStation* ds = DS_DriverStation_new_team(4069, alliance); // alliance is now invalid
    DS_DriverStation_set_tcp_consumer(ds, &callback);
    DS_DriverStation_set_mode(ds, Teleoperated);
    DS_DriverStation_enable(ds);
    while(1) {}
}
