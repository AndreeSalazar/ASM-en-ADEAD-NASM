module.exports = grammar({
  name: 'adead',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  word: $ => $.identifier,

  // Declarar conflictos explícitos - Tree-sitter los resolverá usando precedencia
  conflicts: $ => [
    [$.while_statement, $.struct_literal],
    [$.if_statement, $.struct_literal],
  ],

  rules: {
    // Root: Programa completo
    source_file: $ => repeat($._statement),

    // Statements (declaraciones)
    // IMPORTANTE: while_statement e if_statement deben estar ANTES de expression_statement
    // para evitar que se parseen como expresiones
    _statement: $ => choice(
      $.while_statement,
      $.if_statement,
      $.print_statement,
      $.let_statement,
      $.function_definition,
      $.return_statement,
      $.assign_statement,
      $.expression_statement,
    ),

    // Print statement
    print_statement: $ => seq(
      token('print'),  // Keyword
      $._expression,
    ),

    // Let statement (declaración de variable)
    let_statement: $ => seq(
      token('let'),  // Keyword
      optional(token('mut')),
      $.identifier,
      '=',
      $._expression,
    ),

    // Assign statement (asignación a variable existente)
    assign_statement: $ => seq(
      $.identifier,
      '=',
      $._expression,
    ),

    // While statement (loop)
    // CRÍTICO: Precedencia 10 (muy alta) para evitar conflictos con struct_literal
    // El problema: 'while i <= max {' se parsea como 'i <= (max { ... })'
    // Solución: Dar precedencia 10 a while_statement vs 9 de struct_literal
    while_statement: $ => prec(10, seq(
      'while',
      $._expression,
      $.block,
    )),

    // If statement (condicional) - misma precedencia alta
    if_statement: $ => prec(10, seq(
      'if',
      $._expression,
      $.block,
      optional($.else_clause),
    )),

    else_clause: $ => seq(
      'else',
      choice($.block, $.if_statement), // else if
    ),

    // Block (bloque de código)
    block: $ => seq(
      '{',
      repeat($._statement),
      '}',
    ),

    // Function definition
    function_definition: $ => seq(
      optional('pub'),
      'fn',
      $.identifier,
      '(',
      optional($.parameter_list),
      ')',
      $.block,
    ),

    parameter_list: $ => seq(
      $.identifier,
      repeat(seq(',', $.identifier)),
    ),

    // Return statement
    return_statement: $ => seq(
      token('return'),  // Keyword
      optional($._expression),
    ),

    // Expression statement (expresión como statement)
    expression_statement: $ => $._expression,

    // Expressions (expresiones)
    _expression: $ => choice(
      $.binary_expression,
      $.unary_expression,
      $.function_call,
      $.array_access,
      $.field_access,
      $.primary_expression,
    ),

    // Binary expressions (con precedencia)
    // Precedencia: || < && < == != < <= > >= < + - < * / %
    binary_expression: $ => {
      const prec_table = [
        [1, ['||']],
        [2, ['&&']],
        [3, ['==', '!=', '<', '<=', '>', '>=']],
        [4, ['+', '-']],
        [5, ['*', '/', '%']],
      ];

      return choice(...prec_table.flatMap(([prec_level, operators]) => 
        operators.map(op => prec.left(
          prec_level,
          seq($._expression, op, $._expression),
        ))
      ));
    },

    // Unary expressions
    unary_expression: $ => prec.left(6, seq(
      choice('!', '-'),
      $._expression,
    )),

    // Primary expressions
    // IMPORTANTE: struct_literal debe estar al final para que while/if statements
    // tengan prioridad cuando aparecen keywords
    primary_expression: $ => choice(
      $.number,
      $.float,
      $.string,
      $.boolean,
      $.identifier,
      $.parenthesized_expression,
      $.array_literal,
      $.struct_literal,
    ),

    // Parenthesized expression
    parenthesized_expression: $ => seq(
      '(',
      $._expression,
      ')',
    ),

    // Function call (mayor precedencia que primary_expression)
    function_call: $ => prec(8, seq(
      $.identifier,
      '(',
      optional($.argument_list),
      ')',
    )),

    argument_list: $ => seq(
      $._expression,
      repeat(seq(',', $._expression)),
    ),

    // Array access
    array_access: $ => prec(7, seq(
      $._expression,
      '[',
      $._expression,
      ']',
    )),

    // Field access
    field_access: $ => prec(7, seq(
      $._expression,
      '.',
      $.identifier,
    )),

    // Array literal
    array_literal: $ => seq(
      '[',
      optional($.argument_list),
      ']',
    ),

    // Struct literal - precedencia 9 (menor que while/if que tienen 10)
    struct_literal: $ => prec(9, seq(
      $.identifier,
      '{',
      optional($.field_initializer_list),
      '}',
    )),

    field_initializer_list: $ => seq(
      $.field_initializer,
      repeat(seq(',', $.field_initializer)),
    ),

    field_initializer: $ => seq(
      $.identifier,
      ':',
      $._expression,
    ),

    // Literals
    number: $ => /[0-9]+/,
    float: $ => /[0-9]+\.[0-9]+(e[+-]?[0-9]+)?|[0-9]+\.|\.?[0-9]+/,
    string: $ => seq('"', repeat(choice(/[^"\\]/, $.escape_sequence)), '"'),
    boolean: $ => choice('true', 'false'),
    // Identifier - pero 'while', 'if', 'let', 'print', 'fn', 'return', 'pub' son keywords
    // Tree-sitter los manejará automáticamente si están en seq() como strings
    identifier: $ => token(choice(
      /[a-zA-Z_][a-zA-Z0-9_]*/,
      // Excluir keywords explícitamente - pero Tree-sitter ya lo hace si los usamos como strings
    )),

    // Escape sequences in strings
    escape_sequence: $ => seq(
      '\\',
      choice(
        /[nrt\\"]/,
        /u[0-9a-fA-F]{4}/,
      ),
    ),

    // Comments
    comment: $ => token(seq(
      '//',
      /.*/,
    )),
  },
});

