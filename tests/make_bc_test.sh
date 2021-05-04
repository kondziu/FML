number=$1
test=$2

./fml.sh parse "bc_test_$number/$test.fml" -o "bc_test_$number/$test.json"
./fml.sh compile "bc_test_$number/$test.json" -o "bc_test_$number/$test.bc"
./fml.sh disassemble "bc_test_$number/$test.bc" | tee > "bc_test_$number/$test.bc.txt"
./fml.sh execute "bc_test_$number/$test.bc" | xargs -I{} echo "// >" {} | tee >> "bc_test_$number/$test.bc.txt"

