SELECT
	table_name,
	column_name,
	ordinal_position,
	is_nullable,
	data_type,
	numeric_precision,
	numeric_scale,
	CASE WHEN data_type = 'ARRAY' THEN
		substr(udt_name, 2)
	END AS inner_type
FROM
	information_schema.columns
WHERE
	table_schema = $1
ORDER BY
	table_name,
	ordinal_position

