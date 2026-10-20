local native, genv = ...

local info = debug.info
local newcclosure = native.newcclosure
local version = native.version

local function export(functions)
	for name, f in functions do
		genv[name] = if info(f, "s") == "[C]" then f else newcclosure(f)
	end
end

export({
	newcclosure = newcclosure,
	loadstring = native.loadstring,
	getthreadidentity = native.getthreadidentity,
	setthreadidentity = native.setthreadidentity,
	getidentity = native.getthreadidentity,
	setidentity = native.setthreadidentity,
	getthreadcontext = native.getthreadidentity,
	setthreadcontext = native.setthreadidentity,
	getgenv = function()
		return genv
	end,
	identifyexecutor = function()
		return "Sunlight", version
	end,
})
genv.getexecutorname = genv.identifyexecutor
