#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * The error value returned by joystick functions if the specified port is out of bounds.
 */
#define EOUTOFBOUND 1

/**
 * The error value returned by joystick functions if the joysticks Mutex was poisoned
 */
#define EPOISONLOCK 2

/**
 * The mask for Autonomous mode being selected
 */
#define TRACE_AUTONOMOUS 4

/**
 * The mask for the robot being disabled
 */
#define TRACE_DISABLED 1

/**
 * The mask for the target being a roboRIO
 */
#define TRACE_IS_ROBORIO 16

/**
 * The mask for robot code being alive
 */
#define TRACE_ROBOT_CODE 32

/**
 * The mask for Teleop mode being selected
 */
#define TRACE_TELEOP 2

/**
 * The mask for Test mode being selected
 */
#define TRACE_TEST_MODE 8

typedef enum {
  Normal,
  Simulation,
} DsMode;

typedef enum {
  Autonomous,
  Teleoperated,
  Test,
} Mode;

/**
 * Struct abstracting the byte value for alliance colour and position
 */
typedef struct Alliance Alliance;

/**
 * Represents a connection to the roboRIO acting as a driver station
 *
 * This struct will contain relevant functions to update the state of the robot,
 * and also manages the threads that manage network connections and joysticks
 */
typedef struct DriverStation DriverStation;

typedef struct {
  const char *message;
} StdoutMessage;

/**
 * Constructs a new Alliance representing a Blue alliance robot of the given position
 */
Alliance *DS_Alliance_new_blue(uint8_t position);

/**
 * Constructs a new Alliance representing a Red alliance robot of the given position
 */
Alliance *DS_Alliance_new_red(uint8_t position);

/**
 * Returns the reported battery voltage of the connected robot
 *
 * This function returns 0F if the given pointer is NULL, otherwise it returns the reported battery voltage
 * If no robot is connected this function will return 0F.
 */
float DS_DriverStation_battery_voltage(const DriverStation *ds);

/**
 * Safely frees a given DriverStation.
 *
 * This function should only be passed pointers that were allocated via DS_DriverStation_new_team or DS_DriverStation_new_ip
 */
void DS_DriverStation_destroy(DriverStation *ds);

/**
 * Disables the robot connected to the given ds
 *
 * This function does nothing if ds is NULL
 */
void DS_DriverStation_disable(DriverStation *ds);

/**
 * Enables the robot connected to the given ds
 *
 * This function does nothing if ds is NULL
 */
void DS_DriverStation_enable(DriverStation *ds);

/**
 * Checks whether the given DS is enabling its connected robot
 *
 * This function returns false if the pointer is NULL, and the true/false depending on whether the robot is enabled otherwise
 */
bool DS_DriverStation_enabled(const DriverStation *ds);

/**
 * Emergency stops the robot connected to the given ds
 *
 * This function does nothing if ds is NULL
 */
void DS_DriverStation_estop(DriverStation *ds);

/**
 * Checks whether the given ds is estopping its connected robot
 *
 * This function returns false if ds is NULL, and the status reported by the driver station otherwise.
 */
bool DS_DriverStation_estopped(const DriverStation *ds);

/**
 * Gets the DsMode of the specified ds, DsMode can specify whether the DS is currently connected to a simulator
 *
 * This function returns 1 if either pointer is NULL, and 0 on a success
 * On a successful function call, the value of `mode` will be updated with the current DsMode of the driver station.
 */
uint8_t DS_DriverStation_get_ds_mode(const DriverStation *ds,
                                     DsMode *mode);

/**
 * Gets the robot mode of the specified ds, updating the value in `mode`
 *
 * This function returns 1 if either pointer is NULL, and 0 on a success
 * On a success the value of `mode` will be updated with the current mode of the DS.
 */
uint8_t DS_DriverStation_get_mode(const DriverStation *ds, Mode *mode);

/**
 * Gets the team number currently assigned to the given DriverStation
 *
 * This function will return 0 if the given ds is NULL.
 */
uint32_t DS_DriverStation_get_team_number(const DriverStation *ds);

/**
 * Constructs a new DriverStation that will connect to the specified IP, and that will be assigned the given alliance and team number
 *
 * This function will return NULL if alliance or ip is NULL
 * After calling this function, alliance will no longer be a valid pointer. Attempting to use it may result in UB.
 * The pointer returned by this function **must** be freed using DS_DriverStation_destroy(). Using any other means is undefined.
 */
DriverStation *DS_DriverStation_new_ip(const char *ip,
                                       Alliance *alliance,
                                       uint32_t team_number);

/**
 * Constructs a new DriverStation that will connect to 10.TE.AM.2 with the given team, and that will be assigned the given alliance.
 *
 * This function will return NULL if alliance is NULL
 * After calling this function, alliance will no longer be a valid pointer. Attempting to use it may result in UB.
 * The pointer returned by this function **must** be freed using DS_DriverStation_destroy(). Using any other means is undefined.
 */
DriverStation *DS_DriverStation_new_team(uint32_t team_number,
                                         Alliance *alliance);

/**
 * Instructs the roboRIO connected to the given driver station to restart user code
 *
 * This function does nothing if the given pointer is NULL
 */
void DS_DriverStation_restart_code(DriverStation *ds);

/**
 * Instructs the roboRIO connected to the given driver station to reboot itself
 *
 * This function does nothing if the given pointer is NULL
 */
void DS_DriverStation_restart_roborio(DriverStation *ds);

/**
 * Assigns the given alliance station to the given driver station
 *
 * This function does nothing if ds or alliance are NULL
 * After calling this function, the alliance pointer will no longer be valid.
 */
void DS_DriverStation_set_alliance(DriverStation *ds, Alliance *alliance);

/**
 * Updates the Game Specific Message (GSM) associated with the given DriverStation.
 *
 * This is additional information that can be provided to robot code by the DS, such as colour information in 2020,
 * or switch/scale assignments in 2018.
 *
 * This function will return -1 if either of the given pointers are null
 * It will return 1 if there was an error in the Rust code updating the GSM
 * It will return 0 on a success.
 */
int8_t DS_DriverStation_set_game_specific_message(DriverStation *ds,
                                                  const char *message);

/**
 * Changes the robot mode of the specified ds
 *
 * If ds is NULL, this function does nothing.
 */
void DS_DriverStation_set_mode(DriverStation *ds, Mode mode);

/**
 * Register a callback to be notified when the driver station returns TCP packets containing riolog data
 *
 * This function does nothing if the given ds pointer is NULL
 *
 * WARNING: The pointer passed to the callback is INVALIDATED after the callback returns
 * If keeping the string is desirable, it should be copied out of the pointer provided.
 * Keeping the raw pointer after the callback returns will result in a use-after-free bug when it
 * is next accessed.
 */
void DS_DriverStation_set_tcp_consumer(DriverStation *ds,
                                       void (*callback)(StdoutMessage));

/**
 * Updates the team number of the given driver station. This will automatically reconnect the
 * network threads to target 10.TE.AM.2
 *
 * This function does nothing if ds is NULL
 */
void DS_DriverStation_set_team_number(DriverStation *ds, uint32_t team_number);

/**
 * Specifies whether the driver station should attempt to connect to 172.22.11.2 over USB rather than any other specified target
 *
 * This function does nothing if ds is NULL
 */
void DS_DriverStation_set_use_usb(DriverStation *ds,
                                  bool use_usb);

/**
 * Returns the latest Trace returned by the roboRIO connected to the given driver station
 *
 * Trace is a bitflags value, the individual bitmasks are #define'd at the top of the header.
 *
 * This function does nothing if the given pointer is NULL
 */
uint8_t DS_DriverStation_trace(const DriverStation *ds);

/**
 * Attaches a new joystick, creating the new vector for it.
 * After calling this function, `port` can be used in the set_* functions to update values from the joystick
 *
 * Returns:
 * `EOUTOFBOUND` if the specified port is greater than 5 (RIO only supports 6 joysticks)
 * `EPOISONLOCK` if the Mutex that stores the joysticks data was poisoned.
 * 0 if the operation was a success.
 */
uint8_t DS_Joystick_attach(uintptr_t port);

/**
 * Detaches a joystick, removing all its entries from the DS
 * After calling this function, `port` should **not** be used with set_* functions
 * If there are joysticks bound to ports greater than that specified, the vector may not be deleted,
 * however its contents will be cleared.
 *
 * Returns:
 * `EOUTOFBOUND` if the specified port is greater than 5. (RIO only supports 6 joysticks).
 * `EPOISONLOCK` if the Mutex that stores the joystick data was poisoned.
 * 0 if the operation was a success
 */
uint8_t DS_Joystick_detach(uintptr_t port);

/**
 * Initializes the joystick supplier for the given DriverStation
 * After this is called, joystick values set with this API will be sent to any connected roboRIOs.
 *
 * This function should only be called with a pointer returned from `DS_DriverStation_new_team` or `DS_DriverStation_new_ip`.
 *
 * Returns:
 * -1 if the given pointer is NULL
 * 0 if the operation was a success.
 */
int8_t DS_Joystick_init(DriverStation *ds);

/**
 * Updates the value of an axis associated with the joystick on port `port`
 * This function should only be used if `port` has been registered with `DS_Joystick_attach`
 *
 * Returns:
 * `EOUTOFBOUND` if there is no vector stored at index `port`
 * `EPOISONLOCK` if the Mutex that stores joystick data was poisoned
 */
uint8_t DS_Joystick_set_axis(uintptr_t port, uint8_t axis, float value);

/**
 * Updates the value of a button associated with the joystick on port `port`.
 * This function should only be used if `port` has been registered with `DS_Joystick_attach`
 *
 * Returns:
 * `EOUTOFBOUND` if there is no vector stored at index `port`
 * `EPOISONLOCK` if the Mutex that stores joystick data was poisoned.
 * 0 if the operation was a success
 */
uint8_t DS_Joystick_set_button(uintptr_t port, uint8_t button, bool pressed);

/**
 * Updates the value of a POV, or d-pad associated with the joystick on port `port`
 * This function should only be used if `port` has been registered with `DS_Joystick_attach`
 *
 * Returns:
 * `EOUTOFBOUND` if there is no vector stored at index `port`
 * `EPOISONLOCK` if the Mutex that stores joystick data was poisoned
 */
uint8_t DS_Joystick_set_pov(uintptr_t port, uint8_t pov, int16_t value);
