import argparse
import sdmm_python

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Finds the given /decl/recipe/ child class and prints its recipe.")
    parser.add_argument("dme", type=str, help="Path of the .dme file to parse.")
    parser.add_argument("recipe", type=str, help="The specific class to inspect.")

    args = parser.parse_args()

    assert args.recipe.startswith("/decl/recipe/"), "Recipe type path must begin with /decl/recipe/"

    print("Parsing project.")
    tree = sdmm_python.DmObjectTree(args.dme)

    recipe = tree.get_path(args.recipe)
    vars_to_inspect = ["items", "reagents", "fruit"]

    print(f"Recipe for: {recipe.path}:")
    result = recipe.vars["result"]
    print(f"\tResult: {result.value_repr()}")

    for var_name in vars_to_inspect:
        if not recipe.overrides_variable(var_name):
            continue

        var = recipe.vars[var_name]
        print(f"\t{var_name}: {var.value_repr()}")
