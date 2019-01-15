#ifndef __OPCODES_H__
#define __OPCODES_H__


enum Opcodes {
	FoldNull,
	FoldTrue,
	FoldFalse,
	FoldThis,
	FoldInt,
	FoldStack,
	FoldGlobal,
	FoldEnv,
	FoldField,
	FoldArray,
	FoldIndex,
	FoldBuiltin,
	SetStack,
	SetGlobal,
	SetEnv,
	SetField,
	SetArray,
	SetIndex,
	SetThis,
	Push,
	Pop,
	Call,
	ObjCall,
	Jump,
	JumpIf,
	JumpIfNot,
	Trap,
	EndTrap,
	Ret,
	MakeEnv,
	MakeArray,
	Bool,
	IsNull,
	IsNotNull,
	Add,
	Sub,
	Mult,
	Div,
	Mod,
	Shl,
	Shr,
	UShr,
	Or,
	And,
	Xor,
	Eq,
	Neq,
	Gt,
	Gte,
	Lt,
	Lte,
	Not,
	TypeOf,
	Compare,
	OP_Hash,
	New,
	JumpTable,
	Apply,
	FoldStack0,
	FoldStack1,
	FoldIndex0,
	FoldIndex1,
	PhysCompare,
	TailCall,
	Loop,

	MakeArray2,
	FoldInt32,
	Last,
};


#ifdef PARAMETER_TABLE
static int parameter_table[] = {
	0, // FoldNull
	0, // FoldTrue
	0, // FoldFalse
	0, // FoldThis
	1, // FoldInt
	1, // FoldStack
	1, // FoldGlobal
	1, // FoldEnv
	1, // FoldField
	0, // FoldArray
	1, // FoldIndex
	1, // FoldBuiltin
	1, // SetStack
	1, // SetGlobal
	1, // SetEnv
	1, // SetField
	0, // SetArray
	1, // SetIndex
	0, // SetThis
	0, // Push
	1, // Pop
	1, // Call
	1, // ObjCall
	1, // Jump
	1, // JumpIf
	1, // JumpIfNot
	1, // Trap
	0, // EndTrap
	1, // Ret
	1, // MakeEnv
	1, // MakeArray
	0, // Bool
	0, // IsNull
	0, // IsNotNull
	0, // Add
	0, // Sub
	0, // Mult
	0, // Div
	0, // Mod
	0, // Shl
	0, // Shr
	0, // UShr
	0, // Or
	0, // And
	0, // Xor
	0, // Eq
	0, // Neq
	0, // Gt
	0, // Gte
	0, // Lt
	0, // Lte
	0, // Not
	0, // TypeOf
	0, // Compare
	0, // Hash
	0, // New
	1, // JumpTable
	1, // Apply
	0, // FoldStack0
	0, // FoldStack1
	0, // FoldIndex0
	0, // FoldIndex1
	0, // PhysCompare
	1, // TailCall
	0, // Loop
	1, // MakeArray2
	1, // FoldInt32
};
#endif

#ifdef STACK_TABLE
#define P	0xFF

static int stack_table[] = {
	0, // FoldNull
	0, // FoldTrue
	0, // FoldFalse
	0, // FoldThis
	0, // FoldInt
	0, // FoldStack
	0, // FoldGlobal
	0, // FoldEnv
	0, // FoldField
	-1, // FoldArray
	0, // FoldIndex
	0, // FoldBuiltin
	0, // SetStack
	0, // SetGlobal
	0, // SetEnv
	-1, // SetField
	-2, // SetArray
	-1, // SetIndex
	0, // SetThis
	1, // Push
	-P, // Pop
	-P, // Call
	-P, // ObjCall
	0, // Jump
	0, // JumpIf
	0, // JumpIfNot
	6, // Trap
	-6, // EndTrap
	0, // Ret
	-P, // MakeEnv
	-P, // MakeArray
	0, // Bool
	0, // IsNull
	0, // IsNotNull
	-1, // Add
	-1, // Sub
	-1, // Mult
	-1, // Div
	-1, // Mod
	-1, // Shl
	-1, // Shr
	-1, // UShr
	-1, // Or
	-1, // And
	-1, // Xor
	-1, // Eq
	-1, // Neq
	-1, // Gt
	-1, // Gte
	-1, // Lt
	-1, // Lte
	0, // Not
	0, // TypeOf
	-1, // Compare
	0, // Hash
	0, // New
	0, // JumpTable
	-P, // Apply
	0, // FoldStack0
	0, // FoldStack1
	0, // FoldIndex0
	0, // FoldIndex1
	-1, // PhysCompare
	0, // TailCall
	0, // Loop
	-P, // MakeArray2
	0, // FoldInt32
	0, // Last
};
#endif

#endif // __OPCODES_H__