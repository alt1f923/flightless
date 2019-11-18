def main():
    # EDIT ADD 1 TO X
    x = 5
    # END
    print(x)
# EOF

def edit():
    fin = open(__file__, 'r')
    code_lines = fin.readlines()
    fin.close()

    updated_code_lines = []

    edit = False
    eof = False
    for line in code_lines:
        if not eof:
            if "# EDIT" in line:
                edit = True
                updated_code_lines.append(line)
                continue
            elif "# END" in line:
                edit = False
                updated_code_lines.append(line)
                continue
            elif "# EOF" in line:
                eof = True
                updated_code_lines.append(line)
                continue
            if edit:
                x = int(line.split(' ')[-1].strip()) + 1
                line = f"    x = {x}\n"

        updated_code_lines.append(line)

    code = "".join(updated_code_lines)

    fout = open(__file__, 'w')
    fout.write(code)
    fout.close()

if __name__ == "__main__":
    main()