local native, genv = ...

local info = debug.info
local newcclosure = native.newcclosure
local version = native.version

local function export(functions)
	for name, f in functions do
		genv[name] = if info(f, "s") == "[C]" then f else newcclosure(f)
	end
end

local methods = { GET = true, HEAD = true, POST = true, PUT = true, DELETE = true, OPTIONS = true, PATCH = true }

local request = newcclosure(function(options)
	assert(type(options) == "table", "request: options must be a table")
	local url = options.Url
	assert(type(url) == "string" and url:find("^https?://"), "request: Url must be an http:// or https:// URL")
	local method = string.upper(options.Method or "GET")
	assert(methods[method], `request: unsupported method {method}`)
	assert(options.Body == nil or type(options.Body) == "string", "request: Body must be a string")
	local headers = {}
	for name, value in options.Headers or {} do
		headers[tostring(name)] = tostring(value)
	end
	if options.Cookies then
		local cookies = {}
		for name, value in options.Cookies do
			table.insert(cookies, `{name}={value}`)
		end
		headers.Cookie = table.concat(cookies, "; ")
	end
	return native.request(url, method, headers, options.Body)
end)

local require = require
local renv, registry = native.getrawmetatable(genv).__index, native.getreg()
native.setreadonly(renv, false)
renv._G, renv.shared = registry[2], registry[4]

local instances, key = native.instancecache(), native.instancekey

local function cached(keep)
	local list = {}
	-- iterates a copy, as reading an Instance can cache others mid-traversal
	for k, instance in setmetatable(table.clone(instances), nil) do
		if type(k) == "userdata" and (not keep or keep(instance)) then
			table.insert(list, instance)
		end
	end
	return list
end

local hui = Instance.new("Folder")
hui.Name = game:GetService("HttpService"):GenerateGUID(false)
hui.Parent = game:GetService("CoreGui")

-- these never fire for the iPad app on macOS; check NSRunningApplication.isActive natively if Input needs it
local active = true
local UserInputService = game:GetService("UserInputService")
UserInputService.WindowFocused:Connect(function()
	active = true
end)
UserInputService.WindowFocusReleased:Connect(function()
	active = false
end)

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
	getrenv = function()
		return renv
	end,
	getreg = native.getreg,
	checkcaller = native.checkcaller,
	iscclosure = native.iscclosure,
	islclosure = native.islclosure,
	isexecutorclosure = native.isexecutorclosure,
	clonefunction = native.clonefunction,
	getrawmetatable = native.getrawmetatable,
	setrawmetatable = native.setrawmetatable,
	isreadonly = native.isreadonly,
	setreadonly = native.setreadonly,
	makereadonly = function(table)
		native.setreadonly(table, true)
	end,
	getnamecallmethod = native.getnamecallmethod,
	getinstances = function()
		return cached()
	end,
	getnilinstances = function()
		return cached(function(instance)
			return instance.Parent == nil and instance ~= game
		end)
	end,
	getscripts = function()
		return cached(function(instance)
			return instance:IsA("LocalScript")
				or instance:IsA("ModuleScript")
				or instance:IsA("Script") and instance.RunContext == Enum.RunContext.Client
		end)
	end,
	cloneref = function(instance)
		local k = key(instance)
		local original = instances[k]
		instances[k] = nil
		local clone = native.pushinstance(instance)
		instances[k] = original
		return native.setrawmetatable(clone, native.getrawmetatable(instance))
	end,
	compareinstances = function(a, b)
		return key(a) == key(b)
	end,
	gethui = function()
		return hui
	end,
	isrbxactive = function()
		return active
	end,
	readfile = native.readfile,
	writefile = native.writefile,
	appendfile = native.appendfile,
	isfile = native.isfile,
	isfolder = native.isfolder,
	makefolder = native.makefolder,
	delfile = native.delfile,
	delfolder = native.delfolder,
	listfiles = native.listfiles,
	loadfile = native.loadfile,
	getcustomasset = native.getcustomasset,
	base64encode = native.base64encode,
	base64decode = native.base64decode,
	base64_encode = native.base64encode,
	base64_decode = native.base64decode,
	lz4compress = native.lz4compress,
	lz4decompress = native.lz4decompress,
	identifyexecutor = function()
		return "Sunlight", version
	end,
	request = request,
	http_request = request,
	HttpGet = native.httpget,
	HttpGetAsync = native.httpget,
	require = function(module)
		local ok, result = pcall(require, module)
		if not ok and tostring(result):find("non%-RobloxScript module") then
			local identity = native.getthreadidentity()
			native.setthreadidentity(2)
			ok, result = pcall(require, module)
			native.setthreadidentity(identity)
		end
		return if ok then result else error(result, 0)
	end,
})
genv._G = {}
genv.shared = {}
genv.script = Instance.new("LocalScript")
genv.getexecutorname = genv.identifyexecutor
genv.isourclosure = genv.isexecutorclosure
genv.checkclosure = genv.isexecutorclosure
genv.clonereference = genv.cloneref
genv.get_hidden_gui = genv.gethui
genv.iswindowactive = genv.isrbxactive
genv.isgameactive = genv.isrbxactive
genv.cache = {
	invalidate = newcclosure(function(instance)
		instances[key(instance)] = nil
	end),
	iscached = newcclosure(function(instance)
		return instances[key(instance)] ~= nil
	end),
	replace = newcclosure(function(instance, replacement)
		key(replacement) -- only Instances may enter the cache
		instances[key(instance)] = replacement
	end),
}
genv.http = { request = request }
genv.base64 = { encode = native.base64encode, decode = native.base64decode }
genv.crypt = {
	base64 = genv.base64,
	base64encode = native.base64encode,
	base64decode = native.base64decode,
	base64_encode = native.base64encode,
	base64_decode = native.base64decode,
	lz4compress = native.lz4compress,
	lz4decompress = native.lz4decompress,
}
