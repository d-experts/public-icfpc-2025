#!/usr/bin/env python3

# Problem data: name -> size
PROBLEM_DATA = {
    "probatio": 3,
    "primus": 6,
    "secundus": 12,
    "tertius": 18,
    "quartus": 24,
    "quintus": 30,
}


def get_problem_size(problem_name: str) -> int:
    """
    Get the size for a given problem name.
    
    Args:
        problem_name: The name of the problem
        
    Returns:
        The size of the problem
        
    Raises:
        ValueError: If the problem name is not found
    """
    if problem_name not in PROBLEM_DATA:
        raise ValueError(f"Unknown problem name: {problem_name}")
    
    return PROBLEM_DATA[problem_name]




def list_problems() -> list:
    """
    Get a list of all available problem names.
    
    Returns:
        List of problem names
    """
    return list(PROBLEM_DATA.keys())


def get_all_problems() -> dict:
    """
    Get all problem data.
    
    Returns:
        Dictionary of all problems with their size
    """
    return PROBLEM_DATA.copy()


# Example usage
if __name__ == "__main__":
    # Test the functions
    print("Available problems:")
    for name in list_problems():
        size = get_problem_size(name)
        print(f"  {name}: size={size}")
    
    print("\nTesting get_problem_size:")
    for name in ["probatio", "tertius", "quintus"]:
        size = get_problem_size(name)
        print(f"  {name}: size={size}")
    
    print("\nTesting invalid problem name:")
    try:
        get_problem_size("invalid")
    except ValueError as e:
        print(f"  Error: {e}")