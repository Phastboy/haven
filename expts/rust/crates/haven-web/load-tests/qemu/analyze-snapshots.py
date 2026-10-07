#!/usr/bin/env python3
import csv
import sys
import os

def load_csv(path, key_cols):
    if not os.path.exists(path):
        return {}
    with open(path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        return {tuple(row[k] for k in key_cols): row for row in reader}

def diff_numeric_dicts(start, end, exclude_cols):
    diff = {}
    for k in end:
        if k in exclude_cols or k not in start:
            diff[k] = end[k]
        else:
            try:
                diff[k] = float(end[k]) - float(start[k])
                # Format to int if it's a whole number
                if diff[k].is_integer():
                    diff[k] = int(diff[k])
            except (ValueError, TypeError):
                diff[k] = end[k]
    return diff

def analyze_database(res_dir):
    print("=== Database Stats (haven) ===")
    start = load_csv(f"{res_dir}/pg_stat_database_start.csv", ['datname'])
    end = load_csv(f"{res_dir}/pg_stat_database_end.csv", ['datname'])
    
    db_start = start.get(('haven',), {})
    db_end = end.get(('haven',), {})
    
    if db_start and db_end:
        diff = diff_numeric_dicts(db_start, db_end, ['datid', 'datname'])
        commits = diff.get('xact_commit', 0)
        rollbacks = diff.get('xact_rollback', 0)
        blks_read = diff.get('blks_read', 0)
        blks_hit = diff.get('blks_hit', 0)
        total_blks = blks_read + blks_hit
        hit_ratio = (blks_hit / total_blks * 100) if total_blks > 0 else 100.0
        
        print(f"Commits: {commits}")
        print(f"Rollbacks: {rollbacks}")
        print(f"Blocks Read: {blks_read}")
        print(f"Blocks Hit: {blks_hit}")
        print(f"Cache Hit Ratio: {hit_ratio:.2f}%")
    else:
        print("No haven DB stats found.")
    print()

def analyze_user_tables(res_dir):
    print("=== User Tables (Dead Tuples & Autovacuum) ===")
    start = load_csv(f"{res_dir}/pg_stat_user_tables_start.csv", ['relname'])
    end = load_csv(f"{res_dir}/pg_stat_user_tables_end.csv", ['relname'])
    
    for relname, end_row in end.items():
        start_row = start.get(relname, {})
        diff = diff_numeric_dicts(start_row, end_row, ['relid', 'schemaname', 'relname'])
        
        dead_tuples = end_row.get('n_dead_tup', 0)
        av_runs = diff.get('autovacuum_count', 0)
        
        if int(dead_tuples) > 0 or int(av_runs) > 0:
            print(f"Table: {relname[0]:<20} | Dead Tuples: {dead_tuples:<10} | Autovacuum Runs: {av_runs}")
    print()

def analyze_io(res_dir):
    print("=== IO Stats (Reads) ===")
    start = load_csv(f"{res_dir}/pg_stat_io_start.csv", ['backend_type', 'object', 'context'])
    end = load_csv(f"{res_dir}/pg_stat_io_end.csv", ['backend_type', 'object', 'context'])
    
    for key, end_row in end.items():
        start_row = start.get(key, {})
        diff = diff_numeric_dicts(start_row, end_row, ['backend_type', 'object', 'context'])
        reads = diff.get('reads', 0)
        read_time = diff.get('read_time', 0)
        
        if float(reads) > 0:
            backend, obj, ctx = key
            print(f"{backend}/{obj}/{ctx:<15} | Reads: {reads:<8} | Read Time: {read_time} ms")
    print()

def analyze_statements(res_dir):
    print("=== Top 5 Slowest Queries (By Total Time) ===")
    start = load_csv(f"{res_dir}/pg_stat_statements_start.csv", ['queryid'])
    end = load_csv(f"{res_dir}/pg_stat_statements_end.csv", ['queryid'])
    
    queries = []
    for qid, end_row in end.items():
        start_row = start.get(qid, {})
        diff = diff_numeric_dicts(start_row, end_row, ['userid', 'dbid', 'toplevel', 'queryid', 'query'])
        total_time = diff.get('total_exec_time', diff.get('total_time', 0)) # handle pg12 vs pg13+ naming
        calls = diff.get('calls', 0)
        
        if float(calls) > 0:
            queries.append({
                'query': end_row['query'][:60].replace('\n', ' '),
                'total_time': float(total_time),
                'calls': int(calls),
                'mean_time': float(total_time) / int(calls)
            })
            
    queries.sort(key=lambda x: x['total_time'], reverse=True)
    for q in queries[:5]:
        print(f"{q['total_time']:.2f}ms total | {q['calls']} calls | {q['mean_time']:.2f}ms avg | {q['query']}")
    print()

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: ./analyze-snapshots.py <RESULTS_DIR>")
        sys.exit(1)
        
    res_dir = sys.argv[1]
    analyze_database(res_dir)
    analyze_user_tables(res_dir)
    analyze_io(res_dir)
    analyze_statements(res_dir)
