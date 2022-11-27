# Inputs

**This document is deprecated and slated for removal, please use the Wiki**

This document details the hash structure of "inputs" messages sent out by lxp-bridge. These correspond with transient read-only input registers on the inverter.

The inverter sends these across 3 packets, which are directly mapped into JSON and published in `lxp/{datalog}/inputs/1`, `../2` and `../3`. From lxp-bridge v0.6.0, there is also an `../all` message which combines all three into a single hash of data.

Eventually (not before lxp-bridge v1.0) the individual messages may be removed in favour of the new `all` message. Please prefer use of the `all` message in favour of the 1/2/3 messages in new projects.

If InfluxDB is enabled, these values are sent as a single unified hash which matches the `all` MQTT message.

Example structures are shown below with inline comments.

## 1

```
{
  # Bitfield register of current status - TODO: break out into more useful flags?
  "status": 32,

  # Sum of voltage of PV strings (not sure this is too useful)
  "v_pv": 0.0,
  # Voltage of PV string 1 - only present on hybrid inverters
  "v_pv_1": 0.0,
  # Voltage of PV string 2 - only present on hybrid inverters
  "v_pv_2": 0.0,
  # Voltage of PV string 3 - only present on hybrid inverters
  "v_pv_3": 0.0,

  # Voltage of battery stack
  "v_bat": 49.8,

  # State of Charge of battery pack, in %
  "soc": 61,

  # State of Health of battery pack, in % (may require firmware update, mine isn't populated)
  "soh": 0,

  # Sum of power produced by all PV strings, in W
  "p_pv": 455,
  # Power produced by PV string 1, in W
  "p_pv_1": 455,
  # Power produced by PV string 2, in W
  "p_pv_2": 0,
  # Power produced by PV string 3, in W
  "p_pv_3": 0,

  # Power being used to charge the batteries, in W
  "p_charge": 271,

  # Power from discharging the batteries, in W
  "p_discharge": 0,

  # Voltage of mains AC feed (phase R). For 1-phase, this is the only relevant one
  "v_ac_r": 245.7,
  # Voltage of mains AC feed (phase S). Only relevant for 3-phase, otherwise it tends to float
  "v_ac_s": 15.7,
  # Voltage of mains AC feed (phase T). Only relevant for 3-phase, otherwise it tends to float
  "v_ac_t": 0.0,

  # Frequency of mains AC feed, in Hz
  "f_ac": 49.98,

  "p_inv": 0,
  "p_rec": 277,

  # Power factor of mains AC feed?
  "pf": 1.0,

  # Voltage of EPS (phase R). This is the only one used for 1-phase setups
  "v_eps_r": 245.7,
  # Voltage of EPS (phase S). Only relevant for 3-phase, otherwise it tends to float
  "v_eps_s": 307.2,
  # Voltage of EPS (phase T). Only relevant for 3-phase, otherwise it tends to float
  "v_eps_t": 2875.2,

  # Frequency of EPS, in Hz
  "f_eps": 49.98,

  # Power being exported to grid, in W
  "p_to_grid": 0,

  # Power being imported from grid, in W
  "p_to_user": 0,

  # PV generation, today, in kWh. This is just the sum of 1 to 3 below it
  "e_pv_day": 0.4,
  # PV generation, today, in kWh, of string 1
  "e_pv_day_1": 0.4,
  # PV generation, today, in kWh, of string 2
  "e_pv_day_2": 0.0,
  # PV generation, today, in kWh, of string 3
  "e_pv_day_3": 0.0,

  # Not 100% sure but closely correspond to chg/dischg, so related?
  "e_inv_day": 0.0,
  "e_rec_day": 0.2,

  # Energy put into the batteries, today, in kWh
  "e_chg_day": 0.2,

  # Energy taken out of the batteries, today, in kWh
  "e_dischg_day": 0.0,

  # Energy consumed by the EPS function, today, in kWh
  "e_eps_day": 0.0,

  # Energy exported to the grid, today, in kWh
  "e_to_grid_day": 0.0,

  # Energy imported from the grid, today, in kWh
  "e_to_user_day": 1.9,

  # Internal bus voltages
  "v_bus_1": 376.3,
  "v_bus_2": 304.1,

  # unix timestamp of when this data was received from the inverter
  "time": 1624793103
}
```

## 2

```
{
  # PV generation, alltime, in kWh. This is just the sum of 1 to 3 below it
  "e_pv_all": 3899.8,
  # PV generation, alltime, in kWh, of string 1
  "e_pv_all_1": 3899.8,
  # PV generation, alltime, in kWh, of string 2
  "e_pv_all_2": 0.0,
  # PV generation, alltime, in kWh, of string 3
  "e_pv_all_3": 0.0,

  # Not 100% sure but closely correspond to chg/dischg, so related?
  "e_inv_all": 1476.8,
  "e_rec_all": 1771.4,

  # Energy put into the batteries, alltime, in kWh
  "e_chg_all": 2138.5,

  # Energy taken out of the batteries, alltime, in kWh
  "e_dischg_all": 1829.9,

  # Energy consumed by the EPS function, alltime, in kWh
  "e_eps_all": 0.0,

  # Energy exported to the grid, alltime, in kWh
  "e_to_grid_all": 936.6,

  # Energy imported from the grid, alltime, in kWh
  "e_to_user_all": 2573.7,

  # Temperature inside the inverter, degrees C
  "t_inner": 36,
  # Temperatures of the external radiator, degrees C
  "t_rad_1": 22,
  "t_rad_2": 23,

  # Temperature of the lead-acid battery sensor (if present), degrees C
  "t_bat": 0,

  # Number of seconds the inverter has been running; this does not reset on reboot
  "runtime": 38690201,

  # unix timestamp of when this data was received from the inverter
  "time": 1624793103
}
```


## 3


```
{
  # Maximum charge current, in Amps
  "max_chg_curr": 150.0,

  # Maximum discharge current, in Amps
  "max_dischg_curr": 150.0,

  # Maximum charge voltage? Not quite sure
  "charge_volt_ref": 53.2,

  # Discharge cutoff voltage; some sort of failsafe, usually it shouldn't drop this low
  "dischg_cut_volt": 40.0,

  # Various BMS status flags. These are bitflags so not very useful in this form yet
  # Also see https://github.com/celsworth/lxp-bridge/issues/7
  "bat_status_0": 0,
  "bat_status_1": 0,
  "bat_status_2": 0,
  "bat_status_3": 0,
  "bat_status_4": 0,
  "bat_status_5": 192,
  "bat_status_6": 0,
  "bat_status_7": 0,
  "bat_status_8": 0,
  "bat_status_9": 0,
  "bat_status_inv": 3,

  # Number of batteries in the stack
  "bat_count": 6,

  # Ah capacity of the battery stack
  "bat_capacity": 0,

  # unix timestamp of when this data was received from the inverter
  "time": 1624793103
}
```
