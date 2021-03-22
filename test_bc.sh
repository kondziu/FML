#!/bin/bash
i=0
n=$(find tests -name '*.bc' | wc -l)

for test in tests/*.bc
do
  i=$((i + 1))
  filename="$(dirname "$test")/$(basename "$test" .bc)"
  outfile="$filename.out"
  difffile="$filename.diff"
  txtfile="$filename.bc.txt"

  message=$(echo -n "Executing test [$i/$n]: \"$test\"... ")
  echo -n "$message"

  message_length=$(echo -n "$message" | wc -c)
  for _ in $(seq 1 $((74 - $message_length)))
  do
    echo -n " "
  done

  ./fml.sh execute "$test" 1> "$outfile" 2> "$outfile"

  diff <(grep -e '// >' < "$txtfile" | sed 's/\/\/ > \?//') "$outfile" > "$difffile"
  #grep -e '// >' < "$test" | sed 's/\/\/ > \?//' > "$expectedfile"
  #diff "$expectedfile" "$outfile" > "$difffile"
  if test "$?" -eq 0
  then
    echo -e "\e[32mpassed\e[0m"
  else
    echo -e "\e[31mfailed\e[0m [details \"$difffile\"]"
  fi
done
