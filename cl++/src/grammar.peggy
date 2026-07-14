Program
    = _ decls:FunctionDeclaration* _ {
        return {
            type: "Program", body: decls
        }
    }

FunctionDeclaration
    = "fn" _ name:Identifier _ "()" _ "{" _ body:Statement* _ "}" {
        return {
            type: "FunctionDeclaration",
            name: name.name,
            body: body
        }
    }

Statement
    = VariableDeclaration / ExpressionStatement

VariableDeclaration
 = "var" _ name:Identifier _ "=" _ value:Expression _ {
    return {
            type: "VariableDeclaration",
            name: name.name,
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
    = CallExpression / MemberExpression / PrimaryExpression

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
