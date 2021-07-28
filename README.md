
## Todo

* Do something with NOR in simplifying an expression like !(a or b)
* Might be able to scope fixed vars by block; would need fixed to not just be a boolean
    but an option of a struct with the line numbers for the scope (which would need to be
    adjusted by the mips optimizer when lines are removed)

### (Gas fuel generator)
(posted by .Hunter.)

GFG's need a bit of work to get going, but once you do they are the best power source. For them to run at all, they need a atmosphere of 20kPa or more and to be within this temperature range 5C-55C (278.15K-328.15K). They use fuel to run. If you are new, I'd advise a pressure regulator set rather low, to say 5-10kPa for the fuel input, otherwise you can use math and a volume pump to give it the exact amount of fuel it needs every game tick.

However the biggest thing that always stumps people is keeping the generator cool, as when its on it produces a lot of heat. There are a few options for this, firstly I'd enclose it within a 1x1x1 cube. You can either pump in cooled gas and remove the extra using active vents (with pressure external set to whatever you want the room pressure to be), or you can have a really big radiator array attached to the internal atmosphere. If you want examples of  this:
pumping gas in https://www.youtube.com/watch?v=n7uAweqAEOg&t=601s&ab_channel=Cowsareevil
radiator array https://www.youtube.com/watch?v=ZHq20I1HJ8U&ab_channel=Elmotrix

The other cool thing about GFG's is you can actually fully control how much power they output. Here is some code I wrote to control the fuel input so that the generator outputs the amount of power you want.
https://steamcommunity.com/sharedfiles/filedetails/?id=2509117328

```
R = 8.314
tank_V = 6000 # L
tank_T = 273.15 # K
tank_n0 = 10000 # moles (initial)
tank_n = tank_n0

n_moved = lambda n, r: n * r / tank_V # r: pump ratio

pid = PID.PID(P, I, D)
pid.setSampleTime(0.01)

END = L
feedback = tank_n
target = 500
pid.SetPoint = feedback  - target

feedback_list = []
time_list = []
setpoint_list = []
r_list = []

for i in range(1, END):
    pid.update(feedback)
    
    
    r = max(0, min(100, -pid.output)) # pump ratio
    dn = n_moved(tank_n, r)
    tank_n -= dn
    feedback = tank_n
    r_list.append(r)
    
#     feedback += pid.output
#     feedback += (pid.output - (1/i))

#     output = pid.output
#     if pid.SetPoint > 0:
#         feedback += (output - (1/i))
#     if i > 10:
#         pid.SetPoint = 1
#         pid.SetPoint = 1
    time.sleep(0.02)

    feedback_list.append(feedback)
    setpoint_list.append(pid.SetPoint)
    time_list.append(i)
    
print(r_list)

time_sm = np.array(time_list)
# feedback_smooth = interp.splprep(time_list, feedback_list, time_smooth)
spl = interp.splrep(time_list, feedback_list)
time_smooth = np.linspace(time_sm.min(), time_sm.max(), 300)
feedback_smooth = interp.splev(time_smooth, spl)
```
