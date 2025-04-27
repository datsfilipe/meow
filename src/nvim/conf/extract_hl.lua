local hl_cache = {}
local hl_pool = {}

local function get_or_create_hl()
	return table.remove(hl_pool) or {}
end

local function clear_and_cache_hl(hl, hlid)
	for k in pairs(hl) do
		hl[k] = nil
	end

	---@diagnostic disable-next-line: deprecated
	local hl_data = vim.api.nvim_get_hl_by_id(hlid, true)

	if hl_data.foreground then
		hl.fg = hl_data.foreground
	end

	if hl_data.background then
		hl.bg = hl_data.background
	end

	hl.bold = hl_data.bold or false
	hl.italic = hl_data.italic or false
	hl.underline = hl_data.underline or false

	hl_cache[hlid] = hl
	return hl
end

local function get_highlight_info(hlid)
	if hl_cache[hlid] then
		return hl_cache[hlid]
	end

	local hl = get_or_create_hl()
	return clear_and_cache_hl(hl, hlid)
end

local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
local result = {}

result = table.pack and table.pack(#lines, 0) or {}

for i, line_text in ipairs(lines) do
	local line_data = { text = line_text, segments = {} }
	local last_hl = nil
	local segment_start = 0
	local line_len = #line_text

	if table.pack then
		line_data.segments = table.pack(math.min(line_len, 20), 0)
	end

	for col = 0, line_len do
		local synid = vim.fn.synID(i, col + 1, 1)
		local trans_id = vim.fn.synIDtrans(synid)

		if trans_id ~= 0 then
			if last_hl ~= trans_id then
				if last_hl then
					local text = string.sub(line_text, segment_start + 1, col)
					if text ~= "" then
						table.insert(line_data.segments, {
							text = text,
							hl = get_highlight_info(last_hl),
						})
					end
					segment_start = col
				elseif col > 0 then
					local text = string.sub(line_text, 1, col)
					if text ~= "" then
						table.insert(line_data.segments, {
							text = text,
							hl = {},
						})
					end
					segment_start = col
				end

				last_hl = trans_id
			end
		end
	end

	if last_hl and segment_start < line_len then
		local text = string.sub(line_text, segment_start + 1)
		if text ~= "" then
			table.insert(line_data.segments, {
				text = text,
				hl = get_highlight_info(last_hl),
			})
		end
	elseif line_len > 0 and #line_data.segments == 0 then
		table.insert(line_data.segments, {
			text = line_text,
			hl = {},
		})
	end

	table.insert(result, line_data)
end

hl_cache = {}
for _, line_data in ipairs(result) do
	for _, segment in ipairs(line_data.segments) do
		if segment.hl and next(segment.hl) then
			table.insert(hl_pool, segment.hl)
		end
	end
end

return result
