#!/bin/bash
i=0
n=$(find tests -name '*.fml' | wc -l)

for test in tests/*.fml
do
  i=$((i + 1))
  filename="$(dirname "$test")/$(basename "$test" .fml)"
  outfile="$filename.out"
  difffile="$filename.diff"

  message=$(echo -n "Executing test [$i/$n]: \"$test\"... ")
  echo -n "$message"

  message_length=$(echo -n "$message" | wc -c)
  for _ in $(seq 1 $((74 - $message_length)))
  do
    echo -n " "
  done

  ./fml.sh run "$test" 1> "$outfile" 2> "$outfile"

  diff <(grep -e '// >' < "$test" | sed 's/\/\/ > \?//') "$outfile" > "$difffile"
  if test "$?" -eq 0
  then
    echo -e "\e[32mpassed\e[0m"
  else
    echo -e "\e[31mfailed\e[0m [details \"$difffile\"]"
  fi
done