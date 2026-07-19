Program
    = _ decls:(decl:(ImportDeclaration / FunctionDeclaration) _ { return decl })* {
        return {
            type: "Program", body: decls
        }
    }

FunctionDeclaration
    = pub:("pub" _)? "fn" _ name:Identifier _ "(" _ params:ParameterList _ ")" _ "{" _ body:(stmt:Statement _ { return stmt })* "}" {
        return {
            type: "FunctionDeclaration",
            name: name,
            params: params,
            body: body,
            isPublic: pub !== null
        }
    }

BooleanLiteral
    = "true" {
        return {
            type: "BooleanLiteral",
            value: true
        }
    }
    / "false" {
        return {
            type: "BooleanLiteral",
            value: false
        }
    }

ImportDeclaration
    = "import" _ source:StringLiteral _ {
        return {
            type: "ImportDeclaration",
            source: source
        }
    }


ArrayExpression
    = "[" _ elements:(Expression|.., _ "," _|)? _ "]" {
        return {
            type: "ArrayExpression",
            elements: elements !== null ? elements : []
        }
    }


NumberLiteral
    = [0-9]+ ("." [0-9]+)? {
        return {
            type: "NumberLiteral",
            value: parseFloat(text())
        }
    }

Statement
    = ImportDeclaration / ReturnStatement / IfStatement / WhileStatement / ForInStatement / VariableDeclaration / ExpressionStatement

IfStatement
    = "if" _ "(" _ cond:Expression _ ")" _ "{" _ cons:Statement* _ "}"
    alt:(_ "else" _ (IfStatement / "{" _ Statement* _ "}"))? {
        let alternateNode = undefined
        if (alt) {
            const actualAlt = alt[3]
            if (Array.isArray(actualAlt)) {
                alternateNode = actualAlt[2]
            } else {
                alternateNode = actualAlt
            }
        }
        return {
            type: "IfStatement",
            condition: cond,
            consequent: cons,
            alternate: alternateNode
        }
    }

WhileStatement
    = "while" _ "(" _ cond:Expression _ ")" _ "{" _ body:Statement* _ "}" {
        return {
            type: "WhileStatement",
            condition: cond,
            body: body
        }
    }

TryExpression
    = "try" _ arg:(PostfixExpression) {
        return {
            type: "TryExpression",
            argument: arg
        }
    }

BinaryExpression
    = left:(PostfixExpression) _ op:CompareOperator _ right:Expression {
        return {
            type: "BinaryExpression",
            left: left,
            operator: op,
            right: right
        }
    }

CompareOperator
    = "==" / "~=" / "<=" / ">=" / "<" / ">"

VariableDeclaration
 = "var" _ name:Identifier _ "=" _ value:Expression _ {
    return {
            type: "VariableDeclaration",
            name: name,
            value: value
        }
    }

ExpressionStatement
    = expr:Expression _ {
        return {
            type: "ExpressionStatement",
            expression: expr
        }
    }

Expression
    = LogicalOrExpression

CompareExpression
    = left:AdditiveExpression _ op:CompareOperator _ right:CompareExpression {
        return {
            type: "BinaryExpression",
            left: left,
            operator: op,
            right: right
        }
    }
    / AdditiveExpression

ReturnStatement
    = "return" _ arg:Expression _ {
        return {
            type: "ReturnStatement",
            argument: arg
        }
    }

ForInStatement
    = "for" _ "(" _ iterator:Identifier _ "in" _ iterable:Expression _ ")" _ "{" _ body:(stmt:Statement _ { return stmt })* "}" {
        return {
            type: "ForInStatement",
            iterator: iterator,
            iterable: iterable,
            body: body
        }
    }

AdditiveExpression
    = head:BaseExpression tail:(
        _ op:("++" / "+" / "-") _ right:BaseExpression {
            return {
                operator: op,
                right: right
            }
        }
    )* {
        if (tail.length === 0) return head

        return tail.reduce((result, element) => ({
            type: "BinaryExpression",
            left: result,
            operator: element.operator,
            right: element.right
        }), head)
    }

BaseExpression = TryExpression / PostfixExpression

PostfixExpression
    = head:PrimaryExpression tail:(
        _ "(" _ args:ArgumentList _ ")" {
            return {
                type: "Call",
                args: args
            }
        }
        /
        _ "." _ prop:Identifier {
            return {
                type: "Member",
                property: prop,
                computed: false
            }
        }
        /
        _ "[" _ prop:Expression _ "]" {
            return {
                type: "Member",
                property: prop,
                computed: true
            }
        }
    )* {
        if (tail.length === 0) return head

        return tail.reduce((result, element) => {
            if (element.type === "Call") {
                return {
                    type: "CallExpression",
                    callee: result,
                    arguments: element.args
                }
            } else {
                return {
                    type: "MemberExpression",
                    object: result,
                    property: element.property,
                    computed: element.computed
                }
            }
        }, head)
    }

ArgumentList
    = first:Expression rest:(_ "," _ Expression)* {
        return [first, ...rest.map(r => r[3])]
    } / _ {return []}

AnonymousFunctionExpression
    = "fn" _ "(" _ params:ParameterList? _ ")" _ "{" _ body:(stmt:Statement _ { return stmt })* "}" {
        return {
            type: "AnonymousFunctionExpression",
            params: params !== null ? params : [],
            body: body
        }
    }

ParameterList
    = first:Identifier rest:(_ "," _ Identifier)* {
        return [first, ...rest.map(r => r[3])]
    } / _ {return []}

PrimaryExpression
    = ReceiveExpression / AnonymousFunctionExpression / BooleanLiteral / NumberLiteral / StringLiteral / AtomLiteral / Identifier / ArrayExpression / TupleExpression / Grouping

StringLiteral
    = '"' chars:[^"]* '"' {
        return {
            type: "StringLiteral",
            value: chars.join("")
        }
    }

LogicalOrExpression
    = head:LogicalAndExpression tail:(
        _ "or" _ right:LogicalAndExpression {
            return {
                operator: "or",
                right: right
            }
        }
    )* {
        if (tail.length === 0) return head
        return tail.reduce((result, element) => ({
            type: "LogicalExpression",
            operator: element.operator,
            right: element.right,
            left: result
        }), head)
    }

LogicalAndExpression
    = head:CompareExpression tail:(
        _ "and" _ right:CompareExpression {
            return {
                operator: "and",
                right: right
            }
        }
    )* {
        if (tail.length === 0) return head
        return tail.reduce((result, element) => ({
            type: "LogicalExpression",
            operator: element.operator,
            right: element.right,
            left: result
        }), head)
    }

TupleExpression
    = "(" _ ")" {
        return {
            type: "TupleExpression",
            elements: []
        }
    }
    / "(" _ expr:Expression _ "," _ ")" {
        return {
            type: "TupleExpression",
            elements: [expr]
        }
    }
    / "(" _ first:Expression _ rest:(_ "," _ Expression)+ _ ")" {
        return {
            type: "TupleExpression",
            elements: [first, ...rest.map(r => r[3])]
        }
    }

ReceiveExpression
    = "receive" _ "{" _ cases:(ReceiveCase)* _ "}" {
        return {
            type: "ReceiveExpression",
            cases: cases
        }
    }

ReceiveCase
    = pattern:Expression _ "->" _ "{" _ body:(stmt:Statement _ { return stmt })* "}" {
        return {
            type: "ReceiveCase",
            pattern: pattern,
            body: body
        }
    }

Grouping
    = "(" _ expr:Expression _ ")" {
        return expr
    }

AtomLiteral
    = ":" name:([a-zA-Z_][a-zA-Z0-9_]*) {
        return {
            type: "AtomLiteral",
            value: name[0] + name[1].join("")
        }
    }

Identifier
    = chars:[a-zA-Z_][a-zA-Z0-9_]* {
        return {
            type: "Identifier",
            name: text()
        }
    }

_ "whitespace"
    = ( [ \t\n\r] / "#" [^\n\r]* )*
