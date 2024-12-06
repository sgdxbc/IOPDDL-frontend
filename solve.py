import gurobipy as gp


def main(model_file, print_solution):
    m = gp.read(model_file)
    m.optimize()
    m.write(model_file.replace('.mps', '.sol'))
    if print_solution:
        for v in m.getVars():
            print(f"{v.VarName} {v.X:g}")
        print(f"Obj: {m.ObjVal:g}")


if __name__ == "__main__":
    from sys import argv
    from os import environ

    model_file = dict(enumerate(argv)).get(1, "example.mps")
    print_solution = environ.get("PRINT_SOL", "no") == "yes"
    main(model_file, print_solution)
