#include "maths/evaluator.hpp"

// So this is definitely the worst way of creating a data structure in C
// But whatever! It works
#define STACK_LENGTH(_stack) (_stack##_length)
#define STACK_PUSH(_stack, _value) { _stack[_stack##_length] = (_value); _stack##_length++; }
#define STACK_POP(_stack) (_stack##_length--, _stack[_stack##_length])
#define STACK_PEEK(_stack) (_stack[_stack##_length - 1])

#define OUTPUT_LAST (output[*output_length - 1])
#define OUTPUT_PUSH(value) { output[*output_length] = (value); (*output_length)++; }
#define OUTPUT_PUSH_OPERATOR(o) OUTPUT_PUSH(((struct evaluator_postfix_item){ \
        .is_operator = true, .value = { .op = o } \
    }))
#define OUTPUT_PUSH_NUMBER(num) OUTPUT_PUSH(((struct evaluator_postfix_item){ \
        .is_operator = false, .value = { .number = num } \
    }))

#define DEPTH_INCREASE() { depth++; }
#define DEPTH_DECREASE() { if (depth > 0) { \
    depth--; \
    if (unary_slots[depth] != TOKEN_NONE) { \
        OUTPUT_PUSH_OPERATOR(unary_slots[depth]); \
    } \
    unary_slots[depth] = TOKEN_NONE; \
} }

// Scans through the given token array and changes SUBTRACT tokens to NEGATE
// ones where appropriate.
void evaluator_preprocess_unary(enum token *tokens, token_index_t tokens_length) {
    enum token last_token = TOKEN_NONE;

    for (token_index_t token_index = 0; token_index < tokens_length; token_index++) {
        enum token this_token = tokens[token_index];

        // If the last token was an lparen, another operator, or there just 
        // wasn't one, we treat this as unary if appropriate
        if (last_token == TOKEN_NONE
            || last_token == TOKEN_LPAREN
            || token_is_binop(last_token)) {
            
            if (this_token == TOKEN_PLUS) {
                // Unary plus is a no-op, just stick a NONE here
                tokens[token_index] = TOKEN_NONE;
            } else if (this_token == TOKEN_SUBTRACT) {
                // It's a negation
                tokens[token_index] = TOKEN_NEGATE;
            }
        }

        last_token = this_token;
    }
}

// Adapted from algorithm at:
//   https://en.wikipedia.org/wiki/Shunting-yard_algorithm
//
// A key addition is *depth*, which keeps track of our approximate depth into an
// expression if we were to represent it as a parse tree. The depth starts at 0,
// gets incremented by one as we start parsing some new expression, and gets
// decremented once we're done.
// Only something that could stand alone increases the depth - for example, 
// parsing "3" would increment the depth while parsing it, but parsing "+"
// would not. Numerals like "321", which are actually made of multiple tokens,
// only count as one for the purposes of depth.
// Here's how depth would look parsing some simple expressions. The depths shown
// are the depths *after* processing the above token.
//
//   Token:  3 2 + 4 6 <end>       
//   Depth:  1 1 0 1 1   0
//
//   Token:  3 2 + ( 4 6 - 8 2 ) <end>
//   Depth:  1 1 0 1 2 2 1 2 2 1   0
//
// This extra state can be used to implement unary operators. When encountering
// a unary operator, we add it and the current depth to a stack. When the depth
// is *reduced* to the same depth as a unary operator on the stack, this unary 
// operator is removed from the unary stack and added to the output. To ease
// implementation, maybe each possible depth value should have a slot and 
// adjacent unaries are collapsed onto each other?
enum evaluator_status evaluator_shunt(
    enum token *tokens, token_index_t tokens_length,
    struct evaluator_postfix_item *output, token_index_t *output_length
) {
    // Make a copy of the tokens so we can mutate them
    enum token tokens_copy[tokens_length];
    for (token_index_t i = 0; i < tokens_length; i++) {
        tokens_copy[i] = tokens[i];
    }
    tokens = tokens_copy;

    // Preprocess unary operators
    evaluator_preprocess_unary(tokens, tokens_length);

    // Set up pointers and data structures
    token_index_t tokens_index = 0;

    *output_length = 0;

    enum token operator_stack[TOKEN_LIMIT];
    token_index_t operator_stack_length = 0;

    // Arbitrary length which is hopefully big enough
    // TODO: add an error if this is full?
    enum token unary_slots[32] = { TOKEN_NONE };
    uint8_t depth = 0;

    bool last_was_digit = false;

    // Iterate over tokens
    while (tokens_index < tokens_length) {
        enum token this_token = tokens[tokens_index];
        bool this_was_digit = false;

        // If this token is a digit...
        if (this_token >= TOKEN_0 && this_token <= TOKEN_9) {
            // Get the numeric value of this digit
            evaluator_t value = this_token - TOKEN_0;

            // Is the previous value in the output also a digit?
            // (If there exists a previous value)
            if (last_was_digit) {
                // If so, add this digit to continue that numeric literal
                OUTPUT_LAST.value.number *= 10;
                OUTPUT_LAST.value.number += value;
            } else {
                // Otherwise, just push it straight on
                OUTPUT_PUSH_NUMBER(value);

                // Increase the depth
                DEPTH_INCREASE();
            }

            this_was_digit = true;
        }
        // Or, if this token is a left paren...
        else if (this_token == TOKEN_LPAREN) {
            // Push it onto the stack
            STACK_PUSH(operator_stack, TOKEN_LPAREN);

            // Increase depth
            DEPTH_INCREASE();
        }
        // Or, if this token is a right paren...
        else if (this_token == TOKEN_RPAREN) {
            // Pop the operator stack until we empty it or encounter left paren
            while (STACK_LENGTH(operator_stack) > 0) {
                enum token popped_token = STACK_POP(operator_stack);
                if (popped_token == TOKEN_LPAREN) {
                    goto matching_paren_found;
                } else {
                    OUTPUT_PUSH_OPERATOR(popped_token);
                }
            }

            // We emptied the stack and didn't find a matching bracket, that's a
            // syntax error
            return EVALUATOR_STATUS_SYNTAX_ERROR;

            matching_paren_found:;

            // Decrease depth
            DEPTH_DECREASE();
        }
        // Or, if this token is an operator
        else if (token_is_binop(this_token)) {
            // While...
            while (
                // ...there's a token at the top of the operator stack...
                STACK_LENGTH(operator_stack) > 0
                && token_is_binop(STACK_PEEK(operator_stack)) 
                // ...and the stack token has at least the precedence of the
                // one in the input...
                // (NOTE: shortcut because all implemented ops are left-assoc)
                && token_operator_precedence(STACK_PEEK(operator_stack))
                    >= token_operator_precedence(this_token)
                // ...and the stack token isn't an lparen
                && STACK_PEEK(operator_stack) != TOKEN_LPAREN
            ) {
                // Push the token onto the operator stack
                OUTPUT_PUSH_OPERATOR(STACK_POP(operator_stack));
            }

            STACK_PUSH(operator_stack, this_token);
        }
        // Or, if this is a negation unary...
        else if (this_token == TOKEN_NEGATE) {
            // If there's already a negate in the unary slot...
            if (unary_slots[depth] == TOKEN_NEGATE) {
                // Negate the negate!
                unary_slots[depth] = TOKEN_NONE;
            } else {
                // Otherwise, add it to the current unary slot
                unary_slots[depth] = this_token;
            }
        }
        // Or, if this was TOKEN_NONE...
        else if (this_token == TOKEN_NONE) {
            // Do nothing!
            // This is inserted as a placeholder for unary plus
        }
        
        tokens_index++;

        // If a digit just finished, reduce depth
        if (last_was_digit && !this_was_digit) {
            DEPTH_DECREASE();
        }

        last_was_digit = this_was_digit;
    }

    // Reduce depth to 0
    while (depth > 0) {
        DEPTH_DECREASE();
    }

    // Empty the operator stack
    while (STACK_LENGTH(operator_stack) > 0) {
        OUTPUT_PUSH_OPERATOR(STACK_POP(operator_stack));
    }

    return EVALUATOR_STATUS_OK;
}

enum evaluator_status evaluator_evaluate(
    struct evaluator_postfix_item *items, token_index_t items_length,
    evaluator_t *result
) {
    // Special case: no items is 0
    if (items_length == 0) {
        *result = 0;
        return EVALUATOR_STATUS_OK;
    }

    // Set up everything
    evaluator_t stack[TOKEN_LIMIT];
    token_index_t stack_length = 0;

    token_index_t items_index = 0;

    // Iterate over tokens
    while (items_index < items_length) {
        struct evaluator_postfix_item this_item = items[items_index];

        // If this is a number, push it onto the stack
        if (!this_item.is_operator) {
            STACK_PUSH(stack, this_item.value.number);
        }
        // Otherwise, it's an operator, deal with that
        else {
            evaluator_t a, b;

            switch (this_item.value.op) {
            case TOKEN_PLUS:
                a = STACK_POP(stack);
                b = STACK_POP(stack);
                STACK_PUSH(stack, b + a);
                break;
            case TOKEN_SUBTRACT:
                a = STACK_POP(stack);
                b = STACK_POP(stack);
                STACK_PUSH(stack, b - a);
                break;
            case TOKEN_MULTIPLY:
                a = STACK_POP(stack);
                b = STACK_POP(stack);
                STACK_PUSH(stack, b * a);
                break;
            case TOKEN_DIVIDE:
                a = STACK_POP(stack);
                b = STACK_POP(stack);
                STACK_PUSH(stack, b / a);
                break;
            case TOKEN_NEGATE:
                a = STACK_POP(stack);
                STACK_PUSH(stack, -1 * a);
                break;
            default:
                return EVALUATOR_STATUS_SYNTAX_ERROR;
            }
        }

        items_index++;
    }

    // There should only be one item left on the stack
    if (STACK_LENGTH(stack) != 1) {
        return EVALUATOR_STATUS_SYNTAX_ERROR;
    }

    // The result is the only remaining item
    *result = STACK_POP(stack);
    return EVALUATOR_STATUS_OK;
}
