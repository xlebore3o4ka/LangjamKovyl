Program
    = _ decls:FunctionDeclaration* _ {
        return {
            type: "Program", body: decls
        }
    }

FunctionDeclaration
    = "fn" _ name:Identifier _ "(" _ params:ParameterList _ ")" _ "{" _ body:Statement* _ "}" {
        return {
            type: "FunctionDeclaration",
            name: name,
            params: params,
            body: body
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

NumberLiteral
    = [0-9]+ ("." [0-9]+)? {
        return {
            type: "NumberLiteral",
            value: parseFloat(text)
        }
    }

Statement
    = IfStatement / WhileStatement / VariableDeclaration / ExpressionStatement

IfStatement
    = "if" _ "(" _ cond:Expression _ ")" _ "{" _ cons:Statement* _ "}"
    alt:(_ "else" _ (IfStatement / "{" _ Statement* _ "}")) {
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


BinaryExpression
    = left:(CallExpression / MemberExpression / PrimaryExpression) _ op:CompareOperator _ right:Expression {
        return {
            type: "BinaryExpression",
            left: left,
            operator: op,
            right: right
        }
    }

CompareOperator
    = "==" / "~=" / "<" / ">" / "<=" / ">="

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
    = BinaryExpression / CallExpression / MemberExpression / PrimaryExpression

CallExpression
    = callee:(MemberExpression / PrimaryExpression) _ "(" _ args:ArgumentList _ ")" {
        return {
            type: "CallExpression",
            callee: callee,
            arguments: args
        }
    }

ArgumentList
    = first:Expression rest:(_ "," _ Expression)* {
        return [first, ...rest.map(r => r[3])]
    } / _ {return []}

ParameterList
    = first:Identifier rest:(_ "," _ Identifier)* {
        return [first, ...rest.map(r => r[3])]
    } / _ {return []}

MemberExpression
    = obj:PrimaryExpression "." prop:Identifier {
        return {
            type: "MemberExpression",
            object: obj,
            property: prop
        }
    }

PrimaryExpression
    = StringLiteral / Identifier

StringLiteral
    = '"' chars:[^"]* '"' {
        return {
            type: "StringLiteral",
            value: chars.join("")
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
    = [ \t\n\r]*
