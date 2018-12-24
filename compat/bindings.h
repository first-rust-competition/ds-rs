#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

typedef enum {
  Autonomous,
  Teleoperated,
  Test,
} Mode;

typedef struct {
  void *inner;
} Alliance;

typedef struct {
  void *inner;
} DriverStation;

Alliance *Alliance_new_blue(int pos);

Alliance *Alliance_new_red(int pos);

int DriverStation_connected(DriverStation *ptr);

void DriverStation_disable(DriverStation *ptr);

void DriverStation_enable(DriverStation *ptr);

void DriverStation_estop(DriverStation *ptr);

void DriverStation_free(DriverStation *ptr);

float DriverStation_get_battery_voltage(DriverStation *ptr);

DriverStation *DriverStation_new(unsigned int team_number, Alliance *alliance);

void DriverStation_set_game_specific_message(DriverStation *ptr, const char *gsm);

void DriverStation_set_mode(DriverStation *ptr, Mode mode);
