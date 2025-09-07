interface LeadMsg {
	msg_type: 'set' | 'get' | 'error' | 'bulk';
	cell?: CellRef;
	raw?: string;
	eval?: Eval;
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

// Tagged union
type Eval = { literal: Literal } | { cellref: EvalCellRef } | { range: Range } | 'unset';
