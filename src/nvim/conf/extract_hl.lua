local function get_highlight_info(hlid)
	local result = {}

	---@diagnostic disable-next-line: deprecated
	local hl = vim.api.nvim_get_hl_by_id(hlid, true)

	if hl.foreground then
		result.fg = hl.foreground
	end

	if hl.background then
		result.bg = hl.background
	end

	result.bold = hl.bold or false
	result.italic = hl.italic or false
	result.underline = hl.underline or false

	return result
end

local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
local result = {}

for i, line_text in ipairs(lines) do
	local line_data = { text = line_text, segments = {} }
	local last_hl = nil
	local segment_start = 0

	for col = 0, #line_text do
		local synid = vim.fn.synID(i, col + 1, 1)
		-- local synid = vim.fn.synID(i, col + 1, true)
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

	if last_hl and segment_start < #line_text then
		local text = string.sub(line_text, segment_start + 1)
		if text ~= "" then
			table.insert(line_data.segments, {
				text = text,
				hl = get_highlight_info(last_hl),
			})
		end
	elseif #line_text > 0 and #line_data.segments == 0 then
		table.insert(line_data.segments, {
			text = line_text,
			hl = {},
		})
	end

	table.insert(result, line_data)
end

return result
