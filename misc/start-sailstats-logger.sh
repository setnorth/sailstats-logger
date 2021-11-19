index=0
basename="/home/pi/output/sailstats-logger_output_"
fname="${basename}000.csv"

while [ -e "$fname" ]; do
	printf -v fname '%s%03d.csv' "$basename" "$(( ++index ))"
done

/home/pi/bin/sailstats-logger -o $fname
