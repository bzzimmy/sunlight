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
