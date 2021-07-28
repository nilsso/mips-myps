
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
