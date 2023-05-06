<%inherit file="base.html.mako"/>

<%def name="bits(x)">
    ${'%.2fb'%(-x)}
</%def>

<%!
    import numpy

    AMINO_ACIDS= 'ARNDCQEGHILKMFPSTWYVX'

%>

<%
    def count_occ(target, specific):
        instruction = annotated.plan.instructions[target]
        if instruction.action.type in ('Node', 'Combine'):
            return sum(count_occ(value, specific) for value in instruction.values)
        elif instruction.action.type == 'Sequence':
            return int(position.sequences[int(instruction.action.value)] == AMINO_ACIDS[specific])
%>

<%def name="state(target, specific)">
            % if specific is not None:
                ${bits(executed_values[target][specific])}
            %else:
            <table class="table">
                <tr>
                %for index, value in enumerate(AMINO_ACIDS):
    <td>${value}</td>
                %endfor
                </tr>
               <tr>
                %for index, value in enumerate(executed_values[target]):
                
                    <td style="
                    % if value == numpy.max(executed_values[target]):
                    font-weight: bold;
                    %endif
                    % if index == reference_index:
                    font-style: italic;
                    %endif
                    ">
                    ${bits(value)}</td>
                %endfor
               </tr>
               <tr>
                %for index, value in enumerate(executed_values[target]):
                    <td style="
                    % if value == numpy.max(executed_values[target]):
                    font-weight: bold;
                    %endif
                    % if index == reference_index:
                    font-style: italic;
                    %endif
                    ">
                    ${count_occ(target, index)}</td>
                %endfor
               </tr>
            </table>
            %endif
</%def>

<%def name="show(target, path, specific)">
    <% instruction = annotated.plan.instructions[target] %>
    %if instruction.action.type == 'Node':
    <div style="border: solid 1px black;">
        <div>
            <span class="badge badge-secondary">Node ${instruction.action.value}</span>
            <a clas="btn btn-primary" data-toggle="collapse" href="#collapse-${path}">Open</a>
            ${state(target, specific)}
        </div>
        <div id="collapse-${path}" class="collapse" style="padding-left: 2em">
            %for value in instruction.values:
            ${show(value, path + 'v' + str(value) + 'v', None)}
            %endfor
        </div>
    </div>
    %elif instruction.action.type == 'Combine':
    <div style="border: solid 1px black;">
        <div>
            <span class="badge badge-secondary">Combine</span>
            <a clas="btn btn-primary" data-toggle="collapse" href="#collapse-${path}">Open</a>
            ${state(target, specific)}
        </div>
        <div id="collapse-${path}" class="collapse" style="padding-left: 2em">
            %for value in instruction.values:
            ${show(value, path + 'v' + str(value) + 'v', None)}
            %endfor
        </div>
    </div>
    %elif instruction.action.type == 'Sequence':
    <div style="border: solid 1px black;">
        <div>
            <span class="badge badge-secondary">Sequence ${annotated.sequences[int(instruction.action.value)].id} (${instruction.action.value})</span>
            ${state(target, specific)}
        </div>
    </div>
    %endif
</%def>

<%
reference_index = 'ARNDCQEGHILKMFPSTWYVX'.index(position.reference)
%>

${show(annotated.plan.target, '' + str(annotated.plan.target) , None)}