extends CodeEdit

# Reference to Typst node
@export var typst_node: Typst

func _ready():
	# Copy 'typst_expression' from Typst node to text
	if typst_node != null:
		self.text = typst_node.typst_expression

func _process(_delta):
	# Update 'typst_expression' in Typst node with the current text
	if typst_node != null:
		typst_node.typst_expression = self.text
