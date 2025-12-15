// D import file generated from 'src\adead_metaprog.d'
module adead_metaprog;
import std.stdio;
import std.string;
import std.conv;
enum ExprKind
{
	Number,
	Identifier,
	BinaryOp,
	UnaryOp,
	Call,
}
enum OpType
{
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Eq,
	Ne,
	Lt,
	Le,
	Gt,
	Ge,
}
struct Expr
{
	ExprKind kind;
	string value;
	OpType op;
	Expr* left;
	Expr* right;
	string type;
}
enum GenerateMovASM(string reg, string val) = "    mov " ~ reg ~ ", " ~ val ~ "\n";
template GenerateArithASM(OpType op, string reg1, string reg2)
{
	static if (op == OpType.Add)
	{
		enum GenerateArithASM = "    add " ~ reg1 ~ ", " ~ reg2 ~ "\n";
	}
	else
	{
		static if (op == OpType.Sub)
		{
			enum GenerateArithASM = "    sub " ~ reg1 ~ ", " ~ reg2 ~ "\n";
		}
		else
		{
			static if (op == OpType.Mul)
			{
				enum GenerateArithASM = "    imul " ~ reg1 ~ ", " ~ reg2 ~ "\n";
			}
			else
			{
				static if (op == OpType.Div)
				{
					enum GenerateArithASM = "    mov rcx, " ~ reg2 ~ "\n" ~ "    mov rax, " ~ reg1 ~ "\n" ~ "    cqo\n" ~ "    idiv rcx\n";
				}
				else
				{
					static if (op == OpType.Mod)
					{
						enum GenerateArithASM = "    mov rcx, " ~ reg2 ~ "\n" ~ "    mov rax, " ~ reg1 ~ "\n" ~ "    cqo\n" ~ "    idiv rcx\n" ~ "    mov rax, rdx\n";
					}
					else
					{
						enum GenerateArithASM = "    ; operaci\xc3\xb3n no soportada\n";
					}
				}
			}
		}
	}
}
template GenerateCmpASM(OpType op, string reg1, string reg2)
{
	static if (op == OpType.Eq)
	{
		enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~ "    je .L_equal\n";
	}
	else
	{
		static if (op == OpType.Ne)
		{
			enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~ "    jne .L_not_equal\n";
		}
		else
		{
			static if (op == OpType.Lt)
			{
				enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~ "    jl .L_less\n";
			}
			else
			{
				static if (op == OpType.Le)
				{
					enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~ "    jle .L_less_equal\n";
				}
				else
				{
					static if (op == OpType.Gt)
					{
						enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~ "    jg .L_greater\n";
					}
					else
					{
						static if (op == OpType.Ge)
						{
							enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~ "    jge .L_greater_equal\n";
						}
						else
						{
							enum GenerateCmpASM = "    ; comparaci\xc3\xb3n no soportada\n";
						}
					}
				}
			}
		}
	}
}
pure bool validateExprTypes(Expr* expr);
pure Expr* optimizeExpr(Expr* expr);
string generateExprASM(Expr* expr, ref int labelCounter);
extern (C)
{
	Expr* parseAndValidateExpr(const(char)* source);
	const(char)* generateASMFromExpr(Expr* expr);
	Expr* optimizeExprCTFE(Expr* expr);
	void freeExpr(Expr* expr);
	void freeCString(const(char)* str);
}
extern (C)
{
	void* malloc(ulong size);
	void free(void* ptr);
	void* memcpy(void* dest, void* src, ulong n);
}
