import type { Position } from "./grid.svelte.ts";

interface LeadMsg {
	msg_type: 'set' | 'get' | 'error' | 'bulk' | 'eval';
	cell?: CellRef;
	raw?: string;
	eval?: Eval;
	eval_config?: EvalConfig;
	bulk_msgs?: Array<LeadMsg>;
}

interface CellRef {
	row: number;
	col: number;
}

type LiteralType = 'Number' | 'Boolean' | 'String';
type LiteralValue = number | string | boolean;
type EvalRange = Array<Eval>;

interface Literal {
	type: LiteralType;
	value: LiteralValue;
}

interface EvalCellRef {
	eval: Eval;
	reference: CellRef;
}

interface LeadErr {
	code: 'DivZero' | 'TypeErr' | 'Syntax' | 'Server' | 'Unsupported';
	desc: string;
	title: string;
}

interface EvalConfig {
	do_propagation: boolean;
	force_propagation: boolean;
}

// Tagged union
type Eval =
	| { literal: Literal }
	| { cellref: EvalCellRef }
	| { range: Range }
	| { err: LeadErr }
	| 'unset';

interface CellT {
	raw: string;
	temp_raw: string;
	pos: Position;
	temp_eval?: Eval;
	eval?: Eval;
}

export type { Eval, LeadMsg, LeadErr, Literal, CellRef, LiteralValue, CellT };
