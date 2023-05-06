<%inherit file="base.html.mako"/>

<%def name="bits(x)">
    ${'%.2fb'%(-x)}
</%def>


<table class="table">
<h1>Overall</h1>
    <thead>
        <tr>
            <th></th>
            <th>LHS</th>
            <th>RHS</th>
        </tr>
    </thead>
        <tbody>
    <%! 
        SCORE_PARTS=[
            ('Total', 'total'),
            ('Amino Acid Score', 'amino_acid_score' ), 
            ('Graph Score', 'structure_cost'), 
        ] 
    %>
            % for (title, key) in SCORE_PARTS:
            <tr>
                <th>${title}</th>
                <td>${bits(getattr(lhs.score, key))}</td>                   
                <td>${bits(getattr(rhs.score, key))}</td>
                <td style="background: ${'#40ff00' if getattr(lhs.score, key) < getattr(rhs.score, key) else '#ff0000'}">
                ${bits(getattr(lhs.score, key) - getattr(rhs.score, key))}</td>                   
            </tr>
            % endfor
       </tbody>
</table>
<h1>Amino Acids</h1>
<table class="table">
    <tbody>
        <%
            positions = sorted(enumerate(zip(lhs.positions, rhs.positions)), key = lambda x: -abs(x[1][0].score - x[1][1].score))
        %>
        %for (index, (lhs_position, rhs_position)) in positions:
            <tr>
                <th>${index}</th>
                <td>${bits(lhs_position.score)}</td>
                <td>${bits(rhs_position.score)}</td>

                <td style="background: ${'#40ff00' if lhs_position.score < rhs_position.score else '#ff0000'}">
                ${bits(lhs_position.score - rhs_position.score)}</td>
            </th>
        %endfor
    </tbody>
</table>
</div>