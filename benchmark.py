import os
import time
import subprocess
import statistics
from pathlib import Path
from datetime import datetime

def run_python_conversion(input_file):
    start_time = time.time()
    try:
        result = subprocess.run(
            ['python', 'postman_to_swagger.py', '--input', str(input_file.name)],
            check=True,
            capture_output=True,
            text=True,
            cwd=os.getcwd()
        )
        end_time = time.time()
        return end_time - start_time
    except subprocess.CalledProcessError as e:
        if e.stdout: print(f"stdout: {e.stdout}")
        if e.stderr: print(f"stderr: {e.stderr}")
        return None

def run_rust_conversion(input_file):
    start_time = time.time()
    try:
        result = subprocess.run(
            ['cargo', 'run', '--release', '--', '--input', str(input_file.name)],
            check=True,
            capture_output=True,
            text=True,
            cwd=os.getcwd()
        )
        end_time = time.time()
        return end_time - start_time
    except subprocess.CalledProcessError as e:
        if e.stdout: print(f"stdout: {e.stdout}")
        if e.stderr: print(f"stderr: {e.stderr}")
        return None

def format_time(seconds):
    if seconds is None:
        return "Failed"
    if seconds < 1:
        return f"{seconds*1000:.2f}ms"
    return f"{seconds:.2f}s"

def get_file_size(file_path):
    size_bytes = os.path.getsize(file_path)
    if size_bytes < 1024:
        return f"{size_bytes}B"
    elif size_bytes < 1024*1024:
        return f"{size_bytes/1024:.1f}KB"
    else:
        return f"{size_bytes/(1024*1024):.1f}MB"

def print_stats(title, times):
    if not times:
        return "No successful runs"
    
    mean = statistics.mean(times) * 1000  # Convert to ms
    stdev = statistics.stdev(times) * 1000 if len(times) > 1 else 0
    best = min(times) * 1000
    worst = max(times) * 1000
    
    return (
        f"{title}:\n"
        f"  Best:    {best:.2f}ms\n"
        f"  Worst:   {worst:.2f}ms\n"
        f"  Average: {mean:.2f}ms\n"
        f"  Std Dev: {stdev:.2f}ms"
    )

def truncate_text(text, width):
    """Truncate text to fit within width, adding ... if truncated"""
    if len(text) > width:
        return text[:width-3] + "..."
    return text

def print_summary_box(results):
    # Fixed column widths
    widths = {
        'file': 40,  # Increased width for filename
        'size': 12,
        'python': 15,
        'rust': 15,
        'speedup': 12
    }
    
    # Box drawing characters
    TOP    = "═"
    SIDE   = "║"
    CROSS  = "╬"
    TOP_CROSS = "╦"
    BOTTOM_CROSS = "╩"
    SIDE_CROSS_LEFT = "╠"
    SIDE_CROSS_RIGHT = "╣"
    TOP_LEFT = "╔"
    TOP_RIGHT = "╗"
    BOTTOM_LEFT = "╚"
    BOTTOM_RIGHT = "╝"

    # Total width calculation
    total_width = sum(widths.values()) + len(widths) + 1

    # Print top border
    border = (f"{TOP_LEFT}{TOP * widths['file']}{TOP_CROSS}"
             f"{TOP * widths['size']}{TOP_CROSS}"
             f"{TOP * widths['python']}{TOP_CROSS}"
             f"{TOP * widths['rust']}{TOP_CROSS}"
             f"{TOP * widths['speedup']}{TOP_RIGHT}")
    print(border)

    # Print header
    headers = ["File", "Size", "Python", "Rust", "Speedup"]
    header_row = (f"{SIDE} {headers[0]:<{widths['file']-1}}{SIDE}"
                 f" {headers[1]:<{widths['size']-1}}{SIDE}"
                 f" {headers[2]:<{widths['python']-1}}{SIDE}"
                 f" {headers[3]:<{widths['rust']-1}}{SIDE}"
                 f" {headers[4]:<{widths['speedup']-1}}{SIDE}")
    print(header_row)

    # Print separator
    separator = (f"{SIDE_CROSS_LEFT}{TOP * widths['file']}{CROSS}"
                f"{TOP * widths['size']}{CROSS}"
                f"{TOP * widths['python']}{CROSS}"
                f"{TOP * widths['rust']}{CROSS}"
                f"{TOP * widths['speedup']}{SIDE_CROSS_RIGHT}")
    print(separator)

    # Print data rows
    for filename, size, python_times, rust_times in results:
        # Truncate filename if too long
        trunc_filename = truncate_text(filename, widths['file']-1)
        best_python = format_time(min(python_times)) if python_times else "Failed"
        best_rust = format_time(min(rust_times)) if rust_times else "Failed"
        
        if python_times and rust_times:
            speedup = min(python_times) / min(rust_times)
            speedup_str = f"{speedup:.1f}x"
        else:
            speedup_str = "-"

        row = (f"{SIDE} {trunc_filename:<{widths['file']-1}}{SIDE}"
               f" {size:<{widths['size']-1}}{SIDE}"
               f" {best_python:<{widths['python']-1}}{SIDE}"
               f" {best_rust:<{widths['rust']-1}}{SIDE}"
               f" {speedup_str:<{widths['speedup']-1}}{SIDE}")
        print(row)

    # Print bottom border
    bottom = (f"{BOTTOM_LEFT}{TOP * widths['file']}{BOTTOM_CROSS}"
             f"{TOP * widths['size']}{BOTTOM_CROSS}"
             f"{TOP * widths['python']}{BOTTOM_CROSS}"
             f"{TOP * widths['rust']}{BOTTOM_CROSS}"
             f"{TOP * widths['speedup']}{BOTTOM_RIGHT}")
    print(bottom)

def print_benchmark_results(results):
    print("\n" + "═" * 80)
    print("🚀 BENCHMARK RESULTS")
    print("═" * 80)
    
    for result in results:
        filename, size, python_times, rust_times = result
        
        print(f"\n📄 File: {filename}")
        print(f"📦 Size: {size}")
        print("─" * 80)
        
        # Print Python stats
        print("\n🐍 Python Performance")
        print(print_stats("  Runs", python_times))
        
        # Print Rust stats
        print("\n🦀 Rust Performance")
        print(print_stats("  Runs", rust_times))
        
        # Performance Comparison
        if python_times and rust_times:
            best_python = min(python_times)
            best_rust = min(rust_times)
            speedup = best_python/best_rust
            
            print("\n📊 Performance Comparison")
            print(f"  Speedup: {speedup:.2f}x faster in {'Rust' if speedup > 1 else 'Python'}")
            
            if speedup > 1:
                percent_faster = (speedup - 1) * 100
                print(f"  Rust is {percent_faster:.1f}% faster than Python")
            else:
                percent_faster = (1/speedup - 1) * 100
                print(f"  Python is {percent_faster:.1f}% faster than Rust")
        
        print("\n" + "─" * 80)
    
    # Print summary box at the end
    print_summary_box(results)

def run_benchmarks():
    collections_dir = Path("collections")
    if not collections_dir.exists():
        print("❌ Collections directory not found!")
        return

    results = []
    json_files = list(collections_dir.glob("*.json"))
    
    if not json_files:
        print("❌ No JSON files found in collections directory!")
        return

    print("\n🏃 Running Benchmarks...")
    print(f"⏰ Started at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("═" * 80)

    for file_path in json_files:
        print(f"\n📄 Testing: {file_path.name}")
        file_size = get_file_size(file_path)
        
        python_times = []
        rust_times = []
        
        # Run each implementation 3 times
        for i in range(3):
            print(f"\n▶️  Run {i+1}/3")
            
            # Python run
            print("  🐍 Python: ", end="", flush=True)
            if python_time := run_python_conversion(file_path):
                python_times.append(python_time)
                print(f"✅ ({format_time(python_time)})")
            else:
                print("❌")

            # Rust run
            print("  🦀 Rust:   ", end="", flush=True)
            if rust_time := run_rust_conversion(file_path):
                rust_times.append(rust_time)
                print(f"✅ ({format_time(rust_time)})")
            else:
                print("❌")
        
        results.append((file_path.name, file_size, python_times, rust_times))

    print_benchmark_results(results)
    print(f"\n⏰ Finished at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

if __name__ == "__main__":
    run_benchmarks()